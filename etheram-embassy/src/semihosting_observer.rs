// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::types::{Hash, Height};
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node::observer::{ActionKind, EventLevel, Observer};

pub struct SemihostingObserver {
    level: EventLevel,
}

impl SemihostingObserver {
    pub fn new(level: EventLevel) -> Self {
        Self { level }
    }
}

impl Observer for SemihostingObserver {
    fn min_level(&self) -> EventLevel {
        self.level
    }

    fn node_started(&mut self, peer_id: PeerId) {
        if self.level >= EventLevel::Essential {
            crate::info!("Node {} started", peer_id);
        }
        let _ = peer_id;
    }

    fn message_received(&mut self, peer_id: PeerId, source: &MessageSource) {
        if self.level >= EventLevel::Debug {
            match source {
                MessageSource::Peer(_from) => {
                    crate::info!("Node {} received message from peer {}", peer_id, _from);
                }
                MessageSource::Client(_client_id) => {
                    crate::info!(
                        "Node {} received message from client {}",
                        peer_id,
                        _client_id
                    );
                }
                MessageSource::Timer => {
                    crate::info!("Node {} received timer event", peer_id);
                }
            }
        }
        let _ = (peer_id, source);
    }

    fn context_built(
        &mut self,
        peer_id: PeerId,
        height: Height,
        state_root: Hash,
        pending_tx_count: usize,
    ) {
        if self.level >= EventLevel::Info {
            crate::info!(
                "Node {} context: height={} pending_txs={}",
                peer_id,
                height,
                pending_tx_count
            );
        }
        if self.level >= EventLevel::Trace {
            crate::info!(
                "Node {} state_root=[{} {} {} ...]",
                peer_id,
                state_root[0],
                state_root[1],
                state_root[2]
            );
        }
        let _ = (peer_id, height, state_root, pending_tx_count);
    }

    fn action_emitted(&mut self, peer_id: PeerId, kind: &ActionKind) {
        if self.level >= EventLevel::Debug {
            match kind {
                ActionKind::BroadcastMessage => {
                    crate::info!("Node {} emitted BroadcastMessage", peer_id);
                }
                ActionKind::SendMessage { to } => {
                    crate::info!("Node {} emitted SendMessage to={}", peer_id, to);
                }
                ActionKind::SendClientResponse { client_id } => {
                    crate::info!(
                        "Node {} emitted SendClientResponse client={}",
                        peer_id,
                        client_id
                    );
                }
                ActionKind::UpdateAccount { address } => {
                    crate::info!(
                        "Node {} emitted UpdateAccount addr=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                ActionKind::UpdateCache => {
                    crate::info!("Node {} emitted UpdateCache", peer_id);
                }
                ActionKind::StoreBlock { height } => {
                    crate::info!("Node {} emitted StoreBlock height={}", peer_id, height);
                }
                ActionKind::ExecuteBlock { height } => {
                    crate::info!("Node {} emitted ExecuteBlock height={}", peer_id, height);
                }
                ActionKind::IncrementHeight => {
                    crate::info!("Node {} emitted IncrementHeight", peer_id);
                }
                ActionKind::ScheduleTimeout { event } => {
                    let name = match event {
                        TimerEvent::ProposeBlock => "ProposeBlock",
                        TimerEvent::TimeoutRound => "TimeoutRound",
                    };
                    crate::info!("Node {} emitted ScheduleTimeout({})", peer_id, name);
                }
                ActionKind::UpdateContractStorage { address } => {
                    crate::info!(
                        "Node {} emitted UpdateContractStorage addr=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                ActionKind::TransactionReverted { address } => {
                    crate::info!(
                        "Node {} emitted TransactionReverted from=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                ActionKind::StoreReceipts {
                    height,
                    success_count,
                    out_of_gas_count,
                } => {
                    crate::info!(
                        "Node {} emitted StoreReceipts height={} success={} out_of_gas={}",
                        peer_id,
                        height,
                        success_count,
                        out_of_gas_count
                    );
                }
                ActionKind::Log => {
                    crate::info!("Node {} emitted Log", peer_id);
                }
            }
        }
        let _ = (peer_id, kind);
    }

    fn mutation_applied(&mut self, peer_id: PeerId, kind: &ActionKind) {
        if self.level >= EventLevel::Essential {
            match kind {
                ActionKind::TransactionReverted { address } => {
                    crate::info!(
                        "Node {} mutation TransactionReverted from=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                ActionKind::StoreReceipts {
                    height,
                    success_count,
                    out_of_gas_count,
                } => {
                    crate::info!(
                        "Node {} StoreReceipts height={} success={} out_of_gas={}",
                        peer_id,
                        height,
                        success_count,
                        out_of_gas_count
                    );
                }
                _ => {}
            }
        }
        if self.level >= EventLevel::Info {
            match kind {
                ActionKind::UpdateAccount { address } => {
                    crate::info!(
                        "Node {} mutation UpdateAccount addr=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                ActionKind::StoreBlock { height } => {
                    crate::info!("Node {} mutation StoreBlock height={}", peer_id, height);
                }
                ActionKind::ExecuteBlock { height } => {
                    crate::info!("Node {} mutation ExecuteBlock height={}", peer_id, height);
                }
                ActionKind::IncrementHeight => {
                    crate::info!("Node {} mutation IncrementHeight", peer_id);
                }
                ActionKind::UpdateCache => {
                    crate::info!("Node {} mutation UpdateCache", peer_id);
                }
                ActionKind::UpdateContractStorage { address } => {
                    crate::info!(
                        "Node {} mutation UpdateContractStorage addr=[{} {} ...]",
                        peer_id,
                        address[0],
                        address[1]
                    );
                }
                _ => {}
            }
        }
        let _ = (peer_id, kind);
    }

    fn output_executed(&mut self, peer_id: PeerId, kind: &ActionKind) {
        if self.level >= EventLevel::Info {
            match kind {
                ActionKind::BroadcastMessage => {
                    crate::info!("Node {} output BroadcastMessage", peer_id);
                }
                ActionKind::SendMessage { to } => {
                    crate::info!("Node {} output SendMessage to={}", peer_id, to);
                }
                ActionKind::SendClientResponse { client_id } => {
                    crate::info!(
                        "Node {} output SendClientResponse client={}",
                        peer_id,
                        client_id
                    );
                }
                ActionKind::ScheduleTimeout { event } => {
                    let name = match event {
                        TimerEvent::ProposeBlock => "ProposeBlock",
                        TimerEvent::TimeoutRound => "TimeoutRound",
                    };
                    crate::info!("Node {} output ScheduleTimeout({})", peer_id, name);
                }
                ActionKind::Log => {
                    crate::info!("Node {} output Log", peer_id);
                }
                _ => {}
            }
        }
        let _ = (peer_id, kind);
    }

    fn step_completed(&mut self, peer_id: PeerId, processed: bool) {
        if self.level >= EventLevel::Trace {
            crate::info!("Node {} step completed (processed={})", peer_id, processed);
        }
        let _ = (peer_id, processed);
    }
}
