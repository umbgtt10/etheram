// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::storage::in_memory::in_memory_raft_storage::InMemoryRaftStorage;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message::RaftMessage;
use raft_variants::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use raft_variants::implementations::in_memory_raft_transport::InMemoryRaftTransportState;

pub(super) type OutboxState = EmbassySharedState<Vec<(PeerId, RaftMessage<Vec<u8>>)>>;

pub struct NodeInfraSlot {
    pub(super) storage: InMemoryRaftStorage<Vec<u8>>,
    pub(super) transport_state: EmbassySharedState<InMemoryRaftTransportState<Vec<u8>>>,
    pub(super) timer_state: EmbassySharedState<InMemoryRaftTimerState>,
    pub(super) outbox_state: OutboxState,
}

impl NodeInfraSlot {
    pub(super) fn new() -> Self {
        Self {
            storage: InMemoryRaftStorage::new(),
            transport_state: EmbassySharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new()),
            timer_state: EmbassySharedState::new(InMemoryRaftTimerState::new()),
            outbox_state: EmbassySharedState::new(Vec::new()),
        }
    }
}
