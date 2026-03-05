// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::storage::in_memory::in_memory_storage::InMemoryStorage;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::Address;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::in_memory_timer::InMemoryTimerState;
use etheram_variants::implementations::in_memory_transport::InMemoryTransportState;

pub(super) type OutboxState = EmbassySharedState<Vec<(PeerId, IbftMessage)>>;

pub struct NodeInfraSlot {
    pub(super) storage: InMemoryStorage,
    pub(super) transport_state: EmbassySharedState<InMemoryTransportState<IbftMessage>>,
    pub(super) timer_state: EmbassySharedState<InMemoryTimerState>,
    pub(super) outbox_state: OutboxState,
}

impl NodeInfraSlot {
    pub(super) fn new() -> Self {
        Self {
            storage: InMemoryStorage::new(),
            transport_state: EmbassySharedState::new(InMemoryTransportState::<IbftMessage>::new()),
            timer_state: EmbassySharedState::new(InMemoryTimerState::new()),
            outbox_state: EmbassySharedState::new(Vec::<(PeerId, IbftMessage)>::new()),
        }
    }

    pub(super) fn with_genesis_account(mut self, address: Address, balance: u128) -> Self {
        self.storage = self.storage.with_genesis_account(address, balance);
        self
    }
}
