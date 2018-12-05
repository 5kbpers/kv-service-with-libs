use raft::Config;
use protobuf::{self, Message};

pub fn default_raft_config(id: u64, peers: Vec<u64>) -> Config {
    Config {
        id,
        peers,
        election_tick: 10,
        heartbeat_tick: 1,
        max_size_per_msg: 1024 * 1024 * 1024,
        max_inflight_msgs: 256,
        applied: 0,
        ..Default::default()
    }
}

pub fn parse_data<T: Message>(data: &[u8]) -> T {
    protobuf::parse_from_bytes::<T>(data).unwrap_or_else(|e| {
        panic!("data is corrupted: {:?}", e);
    })
}
