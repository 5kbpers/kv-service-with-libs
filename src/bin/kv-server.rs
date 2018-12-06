extern crate kv;
extern crate clap;
extern crate grpcio;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use clap::{App, Arg};
use kv::server::KvServer;
use kv::kvproto::kvpb_grpc::KvClient;
use grpcio::{EnvBuilder, ChannelBuilder};
use rocksdb::{self, DB, DBOptions};

#[inline]
fn create_client(addr: &str) -> KvClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&addr);
    KvClient::new(ch)
}

#[inline]
fn create_db(path: &str) -> DB {
    let mut opts = DBOptions::new();
    opts.create_if_missing(true);
    DB::open(opts, path).unwrap()
}

fn main() {
    let matches = App::new("KV-Server")
        .arg(
            Arg::with_name("id")
                .short("I")
                .long("id")
                .value_name("ID")
                .help("Set server id(expect 0)")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("addr")
                .short("A")
                .long("addr")
                .value_name("IP:PORT")
                .help("Set server address")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("data-dir")
                .short("s")
                .long("data-dir")
                .value_name("PATH")
                .help("Set the path to store directory")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("peers")
                .short("p")
                .long("peers")
                .alias("peer")
                .value_name("ID=IP:PORT")
                .help("Set raft peers")
                .multiple(true)
                .takes_value(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .value_delimiter(",")
                .long_help("Set raft peers. Use `,` to separate address"),
        ).get_matches();

    println!("start server...");
    let id = matches.value_of("id").unwrap().parse::<u64>().unwrap();
    println!("id = {}", id);
    let mut addr_parts = matches.value_of("addr").unwrap().split(':');
    let host = addr_parts.next().unwrap();
    let port = addr_parts.next().unwrap().parse::<u16>().unwrap();
    println!("port = {}", port);

    let mut peers = HashMap::new();
    if let Some(peers_vec) = matches.values_of("peers") {
        peers_vec
            .map(|s| {
                let mut parts = s.split('=');
                let id = parts.next().unwrap().parse::<u64>().unwrap();
                let addr = parts.next().unwrap();
                peers.insert(id, create_client(addr));
            })
            .count();
    }

    let data_dir = matches.value_of("data-dir").unwrap();
    let store_path = Path::new(data_dir);
    let kv_path = store_path.join(Path::new("db"));
    let raft_path = store_path.join(Path::new("raft"));
    fs::create_dir_all(&kv_path).unwrap_or_default();
    fs::create_dir_all(&raft_path).unwrap_or_default();

    let kvdb = create_db(kv_path.to_str().unwrap());
    KvServer::start_server(id, Arc::new(kvdb), host, port, peers);
}