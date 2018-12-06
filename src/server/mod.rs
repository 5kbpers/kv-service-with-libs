mod util;
mod peer;
mod peer_storage;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, SyncSender, Receiver};

use rocksdb::DB;
use rocksdb::Writable;
use protobuf::Message;
use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink, EnvBuilder, ChannelBuilder};
use raft::eraftpb::{Message as RaftMessage, Entry, ConfChange, EntryType, ConfChangeType};

use super::kvproto;
use super::kvproto::kvpb_grpc::{self, Kv, KvClient};
use super::kvproto::kvrpcpb::*;
use self::peer::PeerMessage;

struct NotifyArgs(u64, String, RespErr);

#[derive(Clone)]
pub struct KvServer {
    id: u64,
    peers: Arc<Mutex<HashMap<u64, KvClient>>>,
    engine: Arc<DB>,
    rf_message_ch: SyncSender<PeerMessage>,
    notify_ch_map: Arc<Mutex<HashMap<u64, SyncSender<NotifyArgs>>>>,
}

impl KvServer {
    pub fn start_server(id: u64, engine: Arc<DB>, host: &str, port: u16) {
        let (apply_sender, apply_receiver) = mpsc::sync_channel(100);
        let rf = peer::Peer::new(id, apply_sender);
        let (rf_sender, rf_receiver) = mpsc::sync_channel(100);
        let (rpc_sender, rpc_receiver) = mpsc::sync_channel(100);
        let mut kv_server = KvServer{
            id, 
            peers: Arc::new(Mutex::new(HashMap::new())), 
            engine,
            rf_message_ch: rf_sender,
            notify_ch_map: Arc::new(Mutex::new(HashMap::new())),
        };
        kv_server.async_rpc_sender(rpc_receiver);
        kv_server.async_applier(apply_receiver);
        let env = Arc::new(Environment::new(10));
        let service = kvpb_grpc::create_kv(kv_server);
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind(host, port)
            .build()
            .unwrap();
        peer::Peer::activate(rf, rpc_sender, rf_receiver);
        server.start();
        for &(ref host, port) in server.bind_addrs() {
            println!("listening on {}:{}", host, port);
        }

        let (tx, rx) = oneshot::channel();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(60));
            }
            tx.send(())
        });
        let _ = rx.wait();
        let _ = server.shutdown().wait();
    }

    fn async_rpc_sender(&mut self, receiver: Receiver<RaftMessage>) {
        let l = self.peers.clone();
        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(m) => {
                        let peers = l.lock().unwrap();
                        let client = peers.get(&m.to);
                        if let Some(c) = client {
                            c.raft(&m).unwrap_or_else(|e| {
                                panic!("send raft msg to {} failed: {:?}", m.to, e);
                            });
                        }
                    },
                    Err(_) => ()
                }
            }
        });
    }

    fn start_op(&mut self, req: &KvReq) -> (RespErr, String) {
        let (sh, rh) = mpsc::sync_channel(0);
        {
            let mut map = self.notify_ch_map.lock().unwrap();
            map.insert(req.get_seq(), sh);
        }
        self.rf_message_ch.send(
            PeerMessage::Propose(
                req.write_to_bytes().unwrap()
            )
        ).unwrap();
        match rh.recv_timeout(Duration::from_millis(1000)) {
            Ok(args) => {
                return (args.2, args.1);
            },
            Err(_) => {
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.remove(&req.get_seq());
                }
                return (RespErr::ErrWrongLeader, String::from(""));
            }
        }
    }

    fn async_applier(&mut self, apply_receiver: Receiver<Entry>) {
        let peers = self.peers.clone();
        let notify_ch_map = self.notify_ch_map.clone();
        let engine = self.engine.clone();
        thread::spawn(move || {
            loop {
                match apply_receiver.recv() {
                    Ok(e) => {
                        let mut result: NotifyArgs;
                        let index;
                        let req: KvReq = util::parse_data(&e.data);
                        index = req.seq;
                        if e.data.len() > 0 {
                            result = Self::apply_entry(e.term, &req, engine.clone());
                        } else {
                            result = NotifyArgs(0, String::from(""), RespErr::ErrWrongLeader);
                        }

                        let mut map = notify_ch_map.lock().unwrap();
                        if let Some(s) = map.get(&index) {
                            s.send(result).unwrap();
                        }
                        map.remove(&index);
                    },
                    Err(_) => (),
                }
            }
        });
    }

    fn apply_entry(term: u64, req: &KvReq, engine: Arc<DB>) -> NotifyArgs {
        match req.req_type {
            ReqType::Get => {
                match engine.get(req.key.as_bytes()) {
                    Ok(op) => {
                        match op {
                            Some(v) => NotifyArgs(term, String::from(v.to_utf8().unwrap()), RespErr::OK),
                            None => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                        }
                    },
                    Err(_) => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                }
            },
            ReqType::Put => {
                match engine.put(req.key.as_bytes(), req.value.as_bytes()) {
                    Ok(_) => NotifyArgs(term, String::from(""), RespErr::OK),
                    Err(_) => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                }
            },
            ReqType::Delete => {
                match engine.delete(req.key.as_bytes()) {
                    Ok(_) => NotifyArgs(term, String::from(""), RespErr::OK),
                    Err(_) => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                }
            }
        }
    }
}

impl Kv for KvServer {
    fn get(&mut self, ctx: RpcContext, req: KvReq, sink: UnarySink<GetResp>) {
        let (err, value) = Self::start_op(self, &req);
        let mut resp = GetResp::new();
        resp.set_err(err);
        resp.set_value(value);
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }

    fn put(&mut self, ctx: RpcContext, req: KvReq, sink: UnarySink<PutResp>) {
        let (err, _) = Self::start_op(self, &req);
        let mut resp = PutResp::new();
        resp.set_err(err);
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }

    fn delete(&mut self, ctx: RpcContext, req: KvReq, sink: UnarySink<DeleteResp>) {
        let (err, _) = Self::start_op(self, &req);
        let mut resp = DeleteResp::new();
        resp.set_err(err);
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }

    fn raft(&mut self, ctx: RpcContext, req: RaftMessage, sink: UnarySink<RaftDone>) {
        self.rf_message_ch.send(PeerMessage::Message(req.clone())).unwrap();
        let resp = RaftDone::new();
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }

    fn raft_conf_change(&mut self, ctx: RpcContext, req: ConfChangeReq, sink: UnarySink<RaftDone>) {
        let cc = req.cc.clone().unwrap();
        if cc.change_type == ConfChangeType::AddNode && cc.change_type == ConfChangeType::AddLearnerNode {
            let mut peers = self.peers.lock().unwrap();
            let env = Arc::new(EnvBuilder::new().build());
            let addr = format!("{}:{}", req.ip, req.port);
            let ch = ChannelBuilder::new(env).connect(&addr);
            peers.insert(cc.node_id, KvClient::new(ch));
        }
        self.rf_message_ch.send(PeerMessage::ConfChange(cc)).unwrap();
        let resp = RaftDone::new();
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }
}