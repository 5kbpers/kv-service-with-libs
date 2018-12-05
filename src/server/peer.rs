use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, SyncSender, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use rocksdb::DB;
use raft::eraftpb::{Entry, EntryType, Message};
use raft::{self, RawNode};

use raft::storage::MemStorage as PeerStorage;
// use super::peer_storage::PeerStorage;
use super::kvproto::kvrpcpb;
use super::util;

pub enum PeerMessage {
    Propose(Vec<u8>),
    Message(Message),
}

pub struct Peer {
    raft_group: RawNode<PeerStorage>,
    last_applying_idx: u64,
    last_compacted_idx: u64,
    apply_ch: SyncSender<Entry>,
}

impl Peer {
    pub fn new(
        id: u64,
        apply_ch: SyncSender<Entry>,
    ) -> Peer {
        let cfg =  util::default_raft_config(id, vec![]);
        let storge = PeerStorage::new();
        Peer {
            raft_group: RawNode::new(&cfg, storge, vec![]).unwrap(),
            last_applying_idx: 0,
            last_compacted_idx: 0,
            apply_ch,
        }
    }

    pub fn activate(mut peer: Peer, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        thread::spawn(move || {
            peer.listen_message(sender, receiver);
        });
    }

    pub fn ready(&mut self) -> raft::Ready {
        self.raft_group.ready()
    }

    fn listen_message(&mut self, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        let mut t = Instant::now();
        let mut timeout = Duration::from_millis(100);
        loop {
            match receiver.recv_timeout(timeout) {
                Ok(PeerMessage::Propose(p)) => {
                    match self.raft_group.propose(vec![], p) {
                        Ok(_) => (),
                        Err(_) => self.apply_message(&Entry::new()),
                    }
                },
                Ok(PeerMessage::Message(m)) => self.raft_group.step(m).unwrap(),
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => return,
            }

            let d = t.elapsed();
            if d >= timeout{
                t = Instant::now();
                timeout = Duration::from_millis(100);
                self.raft_group.tick();
            } else {
                timeout -= d;
            }

            self.on_ready(sender.clone());
        }
    }

    fn on_ready(&mut self, sender: SyncSender<Message>) {
        if !self.raft_group.has_ready() {
            return;
        }

        let mut ready = self.raft_group.ready();
        let is_leader = self.raft_group.raft.leader_id == self.raft_group.raft.id;
        if is_leader {
            let msgs = ready.messages.drain(..);
            for _msg in msgs {
                self.send_message(&_msg);
            }
        }

        if !raft::is_empty_snap(&ready.snapshot) {
            self.raft_group.mut_store()
                .wl()
                .apply_snapshot(ready.snapshot.clone())
                .unwrap()
        }

        if !ready.entries.is_empty() {
            self.raft_group.mut_store().wl().append(&ready.entries).unwrap();
        }

        if let Some(ref hs) = ready.hs {
            self.raft_group.mut_store().wl().set_hardstate(hs.clone());
        }

        if !is_leader {
            let msgs = ready.messages.drain(..);
            for _msg in msgs {
                self.send_message(&_msg);
            }
        }

        if let Some(committed_entries) = ready.committed_entries.take() {
            let mut _last_apply_index = 0;
            for entry in committed_entries {
                // Mostly, you need to save the last apply index to resume applying
                // after restart. Here we just ignore this because we use a Memory storage.
                _last_apply_index = entry.get_index();

                if entry.get_data().is_empty() {
                    // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                    continue;
                }

                if entry.get_entry_type() == EntryType::EntryNormal {
                    self.apply_message(&entry);
                } else if entry.get_entry_type() == EntryType::EntryConfChange {

                }
            }
        }

        // Advance the Raft
        self.raft_group.advance(ready);
    }

    fn send_message(&self, msg: &Message) {
        thread::spawn(move || {
            
        });
    }

    fn apply_message(&self, entry: &Entry) {
        thread::spawn(move || {
            
        });
    }
}
