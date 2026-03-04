// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::types::{Address, Hash, Height};
use crate::incoming::timer::timer_event::TimerEvent;
use etheram_core::types::{ClientId, PeerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventLevel {
    None,
    Essential,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    BroadcastMessage,
    SendMessage {
        to: PeerId,
    },
    SendClientResponse {
        client_id: ClientId,
    },
    UpdateAccount {
        address: Address,
    },
    UpdateCache,
    StoreBlock {
        height: Height,
    },
    ExecuteBlock {
        height: Height,
    },
    IncrementHeight,
    UpdateContractStorage {
        address: Address,
    },
    TransactionReverted {
        address: Address,
    },
    StoreReceipts {
        height: Height,
        success_count: usize,
        out_of_gas_count: usize,
    },
    ScheduleTimeout {
        event: TimerEvent,
    },
    Log,
}

pub trait Observer {
    fn min_level(&self) -> EventLevel;

    fn node_started(&mut self, peer_id: PeerId);

    fn message_received(&mut self, peer_id: PeerId, source: &MessageSource);

    fn context_built(
        &mut self,
        peer_id: PeerId,
        height: Height,
        state_root: Hash,
        pending_tx_count: usize,
    );

    fn action_emitted(&mut self, peer_id: PeerId, kind: &ActionKind);

    fn mutation_applied(&mut self, peer_id: PeerId, kind: &ActionKind);

    fn output_executed(&mut self, peer_id: PeerId, kind: &ActionKind);

    fn step_completed(&mut self, peer_id: PeerId, processed: bool);
}
