// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::common_types::node_role::NodeRole;
use raft_node::observer::RaftActionKind;
use raft_node::observer::RaftEventLevel;
use raft_node::observer::RaftObserver;

pub struct RaftSemihostingObserver {
    level: RaftEventLevel,
}

impl RaftSemihostingObserver {
    pub fn new(level: RaftEventLevel) -> Self {
        Self { level }
    }
}

impl RaftObserver for RaftSemihostingObserver {
    fn min_level(&self) -> RaftEventLevel {
        self.level
    }

    fn node_started(&mut self, peer_id: PeerId) {
        if self.level >= RaftEventLevel::Essential {
            crate::info!("Raft node {} started", peer_id);
        }
        let _ = peer_id;
    }

    fn message_received(&mut self, peer_id: PeerId, source: &MessageSource) {
        if self.level >= RaftEventLevel::Debug {
            match source {
                MessageSource::Peer(from) => {
                    crate::info!("Node {} received message from peer {}", peer_id, from);
                }
                MessageSource::Client(client_id) => {
                    crate::info!(
                        "Node {} received message from client {}",
                        peer_id,
                        client_id
                    );
                }
                MessageSource::Timer => {
                    crate::info!("Node {} received timer event", peer_id);
                }
            }
        }
        let _ = (peer_id, source);
    }

    fn context_built(&mut self, peer_id: PeerId, term: u64, role: NodeRole, log_length: usize) {
        if self.level >= RaftEventLevel::Info {
            crate::info!(
                "Node {} context: term={} role={:?} log_len={}",
                peer_id,
                term,
                role,
                log_length
            );
        }
        let _ = (peer_id, term, role, log_length);
    }

    fn action_emitted(&mut self, peer_id: PeerId, kind: &RaftActionKind) {
        if self.level >= RaftEventLevel::Debug {
            crate::info!("Node {} emitted {:?}", peer_id, kind);
        }
        let _ = (peer_id, kind);
    }

    fn mutation_applied(&mut self, peer_id: PeerId, kind: &RaftActionKind) {
        if self.level >= RaftEventLevel::Debug {
            crate::info!("Node {} mutation {:?}", peer_id, kind);
        }
        let _ = (peer_id, kind);
    }

    fn output_executed(&mut self, peer_id: PeerId, kind: &RaftActionKind) {
        if self.level >= RaftEventLevel::Debug {
            crate::info!("Node {} output {:?}", peer_id, kind);
        }
        let _ = (peer_id, kind);
    }

    fn step_completed(&mut self, peer_id: PeerId, processed: bool) {
        if self.level >= RaftEventLevel::Trace && processed {
            crate::info!("Node {} step completed", peer_id);
        }
        let _ = (peer_id, processed);
    }
}
