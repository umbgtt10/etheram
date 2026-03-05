// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::storage::semihosting::semihosting_storage::SemihostingStorage;
use crate::infra::transport::udp::udp_transport::IbftMessageReceiver;
use crate::infra::transport::udp::udp_transport::UdpInboundTransport;
use crate::infra::transport::udp::udp_transport::UdpOutboundTransport;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::Address;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::implementations::in_memory_transport::InMemoryTransportState;

pub struct NodeInfraSlot {
    pub(super) storage: SemihostingStorage,
    pub(super) inbound_receiver: IbftMessageReceiver,
    pub(super) outbound: UdpOutboundTransport,
    pub(super) timer_state: EmbassySharedState<InMemoryTimerState>,
    pub(super) transport_state: EmbassySharedState<InMemoryTransportState<IbftMessage>>,
}

impl NodeInfraSlot {
    pub(super) fn new(
        peer_id: PeerId,
        inbound: UdpInboundTransport,
        outbound: UdpOutboundTransport,
    ) -> Self {
        Self {
            storage: SemihostingStorage::new(peer_id),
            inbound_receiver: inbound.into_receiver(),
            outbound,
            timer_state: EmbassySharedState::new(InMemoryTimerState::new()),
            transport_state: EmbassySharedState::new(InMemoryTransportState::new()),
        }
    }

    pub(super) fn with_genesis_account(mut self, address: Address, balance: u128) -> Self {
        self.storage = self.storage.with_genesis_account(address, balance);
        self
    }
}
