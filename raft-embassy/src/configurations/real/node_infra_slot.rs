// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::storage::semihosting::semihosting_raft_storage::SemihostingRaftStorage;
use crate::infra::transport::udp::udp_raft_transport::RaftMessageReceiver;
use crate::infra::transport::udp::udp_raft_transport::UdpOutboundRaftTransport;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_variants::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use raft_variants::implementations::in_memory_raft_transport::InMemoryRaftTransportState;

pub struct NodeInfraSlot {
    pub(super) storage: SemihostingRaftStorage<Vec<u8>>,
    pub(super) inbox_receiver: RaftMessageReceiver,
    pub(super) outbound: UdpOutboundRaftTransport,
    pub(super) transport_state: EmbassySharedState<InMemoryRaftTransportState<Vec<u8>>>,
    pub(super) timer_state: EmbassySharedState<InMemoryRaftTimerState>,
}

impl NodeInfraSlot {
    pub(super) fn new(
        peer_id: PeerId,
        inbox_receiver: RaftMessageReceiver,
        outbound: UdpOutboundRaftTransport,
    ) -> Self {
        Self {
            storage: SemihostingRaftStorage::new(peer_id),
            inbox_receiver,
            outbound,
            transport_state: EmbassySharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new()),
            timer_state: EmbassySharedState::new(InMemoryRaftTimerState::new()),
        }
    }
}
