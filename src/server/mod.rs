mod util;
mod peer;
mod peer_storage;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self,  SyncSender, Receiver};

use rocksdb::DB;
use rocksdb::Writable;
use protobuf::Message;
use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink, EnvBuilder, ChannelBuilder};
use raft::eraftpb::{Message as RaftMessage, Entry, EntryType, ConfChange, ConfChangeType};

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
    pub fn start_server(
        id: u64, 
        engine: Arc<DB>, 
        host: &str,
        port: u16, 
        peers: HashMap<u64, KvClient>
    ) {
        let (rf_sender, rf_receiver) = mpsc::sync_channel(100);
        let (rpc_sender, rpc_receiver) = mpsc::sync_channel(100);
        let (apply_sender, apply_receiver) = mpsc::sync_channel(100);

        let peers_id = peers.keys().map(|id| { *id }).collect();
        let rf = peer::Peer::new(id, apply_sender, peers_id);

        let mut kv_server = KvServer{
            id, 
            peers: Arc::new(Mutex::new(peers)), 
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
            .unwrap_or_else(|e| {
                panic!("build server error: {}", e);
            });

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
                        let op = peers.get(&m.to);
                        if let Some(c) = op {
                            let client = c.clone();
                            thread::spawn(move || {
                                client.raft(&m).unwrap_or_else(|e| {
                                    println!("send raft msg to {} failed: {:?}", m.to, e);
                                    RaftDone::new()
                                });
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
            map.insert(req.get_client_id(), sh);
        }
        self.rf_message_ch.send(
            PeerMessage::Propose(
                req.write_to_bytes().unwrap_or_else(|e| {
                    panic!("request write to bytes error: {}", e);
                })
            )
        ).unwrap_or_else(|e| {
            panic!("send propose to raft error: {}", e);
        });
        match rh.recv_timeout(Duration::from_millis(1000)) {
            Ok(args) => {
                return (args.2, args.1);
            },
            Err(_) => {
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.remove(&req.get_client_id());
                }
                return (RespErr::ErrWrongLeader, String::from(""));
            }
        }
    }

    // TODO: check duplicate request.
    fn async_applier(&mut self, apply_receiver: Receiver<Entry>) {
        let notify_ch_map = self.notify_ch_map.clone();
        let engine = self.engine.clone();
        let peers = self.peers.clone();
        thread::spawn(move || {
            loop {
                match apply_receiver.recv() {
                    Ok(e) => {
                        match e.get_entry_type() {
                            EntryType::EntryNormal => {
                                let mut result: NotifyArgs;
                                let req: KvReq = util::parse_data(e.get_data());
                                let index = req.get_client_id();
                                if e.data.len() > 0 {
                                    result = Self::apply_entry(e.term, &req, engine.clone(), peers.clone());
                                    println!("apply_entry: {:?}---{:?}", req, result.2);
                                } else {
                                    result = NotifyArgs(0, String::from(""), RespErr::ErrWrongLeader);
                                    println!("empty_entry: {:?}", req);
                                }
                                let mut map = notify_ch_map.lock().unwrap();
                                if let Some(s) = map.get(&index) {
                                    s.send(result).unwrap_or_else(|e| {
                                        panic!("notify apply result error: {}", e);
                                    });
                                }
                                map.remove(&index);
                            },
                            EntryType::EntryConfChange => {
                                let result = NotifyArgs(0, String::from(""), RespErr::OK);
                                let cc: ConfChange = util::parse_data(e.get_data());
                                let mut map = notify_ch_map.lock().unwrap();
                                if let Some(s) = map.get(&cc.get_node_id()) {
                                    s.send(result).unwrap_or_else(|e| {
                                        panic!("notify apply result error: {}", e);
                                    });
                                }
                                map.remove(&cc.get_node_id());
                            }
                        }
                    },
                    Err(_) => (),
                }
            }
        });
    }

    fn apply_entry(
        term: u64, 
        req: &KvReq, 
        engine: Arc<DB>, 
        peers: Arc<Mutex<HashMap<u64, KvClient>>>
    ) -> NotifyArgs {
        let mut data_key = req.key.clone();
        data_key.insert_str(0, "d_");
        let mut seq_key = "c_".to_owned();
        seq_key.push_str(&req.client_id.to_string());
        if req.req_type != ReqType::Get {
            if let Ok(op) = engine.get(seq_key.as_bytes()) {
                if let Some(s) = op {
                    let max_seq: u64 = s.to_utf8().unwrap().parse().unwrap();
                    if max_seq >= req.seq {
                        return NotifyArgs(term, String::from(""), RespErr::ErrNoKey);
                    }
                }
            }
        }
        match req.req_type {
            ReqType::Get => {
                match engine.get(data_key.as_bytes()) {
                    Ok(op) => {
                        match op {
                            Some(v) => NotifyArgs(term, String::from(v.to_utf8().unwrap()), RespErr::OK),
                            None => NotifyArgs(term, String::from(""), RespErr::ErrNoKey),
                        }
                    },
                    Err(_) => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                }
            },
            ReqType::Put => {
                match engine.put(data_key.as_bytes(), req.value.as_bytes()) {
                    Ok(_) => {
                        engine.put(seq_key.as_bytes(), req.seq.to_string().as_bytes()).unwrap();
                        NotifyArgs(term, String::from(""), RespErr::OK)
                    },
                    Err(e) => {
                        println!("put key error: {}", e);
                        NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader)
                    }
                }
            },
            ReqType::Delete => {
                match engine.delete(data_key.as_bytes()) {
                    Ok(_) => {
                        engine.put(seq_key.as_bytes(), req.seq.to_string().as_bytes()).unwrap();
                        NotifyArgs(term, String::from(""), RespErr::OK)
                    },
                    Err(_) => NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader),
                }
            },
            ReqType::PeerAddr => {
                let mut prs = peers.lock().unwrap();
                let env = Arc::new(EnvBuilder::new().build());
                let ch = ChannelBuilder::new(env).connect(&req.peer_addr);
                prs.insert(req.peer_id, KvClient::new(ch));
                NotifyArgs(term, String::from(""), RespErr::OK)
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
        self.rf_message_ch
            .send(PeerMessage::Message(req.clone()))
            .unwrap_or_else(|e| {
                panic!("send message to raft error: {}", e);
            });
        let resp = RaftDone::new();
        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }

    fn raft_conf_change(&mut self, ctx: RpcContext, req: ConfChangeReq, sink: UnarySink<RaftDone>) {
        let cc = req.cc.clone().unwrap();
        let mut resp = RaftDone::new();
        let mut peer_req = KvReq::new();
        peer_req.set_req_type(ReqType::PeerAddr);
        peer_req.set_peer_addr(format!("{}:{}", req.ip, req.port));
        peer_req.set_peer_id(cc.get_node_id());
        peer_req.set_client_id(cc.get_node_id());
        let (err, _) = self.start_op(&peer_req);
        match err {
            RespErr::OK => {
                let (sh, rh) = mpsc::sync_channel(0);
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.insert(cc.get_node_id(), sh);
                }
                self.rf_message_ch.send(PeerMessage::ConfChange(cc.clone())).unwrap();
                match rh.recv_timeout(Duration::from_millis(1000)) {
                    Ok(_) => resp.set_err(RespErr::OK),
                    Err(_) => resp.set_err(RespErr::ErrWrongLeader),
                }
            },
            _ => resp.set_err(RespErr::ErrWrongLeader),
        }

        ctx.spawn(
            sink.success(resp).map_err(
                move |e| println!("failed to reply {:?}: {:?}", req, e)
            )
        )
    }
}