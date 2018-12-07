extern crate kv;
extern crate rand;
extern crate clap;
extern crate grpcio;

use std::sync::Arc;

use kv::client::Clerk;
use kv::kvproto::kvpb_grpc::KvClient;
use grpcio::{EnvBuilder, ChannelBuilder};
use clap::{App, Arg};

fn create_client(addr: &str) -> KvClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&addr);
    KvClient::new(ch)
}

fn main() {
    let matches = App::new("KV-Client")
        .arg(
            Arg::with_name("servers")
                .short("s")
                .long("servers")
                .alias("server")
                .value_name("IP:PORT")
                .help("Raft servers list")
                .multiple(true)
                .takes_value(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .value_delimiter(",")
                .required(true)
                .long_help("Raft servers list. Use `,` to separate address"),
        ).get_matches();
    let mut servers = vec![];
    if let Some(addr_vec) = matches.values_of("servers") {
        addr_vec.map(|addr| {
            servers.push(create_client(addr));
        })
        .count();
    } 
    let client_id = rand::random();
    let mut client = Clerk::new(&servers, client_id);
    client.put("k", "2333333333333");
    println!("put k");
    println!("value of k is {}", client.get("k"));
}
