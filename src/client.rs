use std::thread;
use std::time::Duration;

use super::kvproto::kvrpcpb::*;
use super::kvproto::kvpb_grpc::KvClient;

pub struct Clerk {
    servers: Vec<KvClient>,
    client_id: u64,
    request_seq: u64,
    leader_id: usize,
}

impl Clerk {
    pub fn new(servers: &Vec<KvClient>, client_id: u64) -> Clerk {
        Clerk {
            servers: servers.clone(),
            client_id,
            request_seq: 0,
            leader_id: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> String {
        let mut req = KvReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Get);
        req.set_key(key.to_owned());
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id].get(&req).unwrap();
            match reply.err {
                RespErr::OK => return reply.value,
                RespErr::ErrWrongLeader => (),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn put(&mut self, key: &str, value: &str) {
        let mut req = KvReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Get);
        req.set_key(key.to_owned());
        req.set_value(value.to_owned());
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id].put(&req).unwrap();
            match reply.err {
                RespErr::OK => return,
                RespErr::ErrWrongLeader => (),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn delete(&mut self, key: &str) {
        let mut req = KvReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Get);
        req.set_key(key.to_owned());
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id].delete(&req).unwrap();
            match reply.err {
                RespErr::OK => return,
                RespErr::ErrWrongLeader => (),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }
}