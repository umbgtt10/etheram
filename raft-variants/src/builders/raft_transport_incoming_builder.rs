// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::in_memory_raft_transport::InMemoryRaftTransport;
use crate::implementations::in_memory_raft_transport::InMemoryRaftTransportState;
use crate::implementations::no_op_raft_transport::NoOpRaftTransport;
use crate::implementations::shared_state::SharedState;
use crate::variants::RaftTransportIncomingVariant;
use alloc::boxed::Box;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::transport_incoming_adapter::TransportIncomingAdapter;

pub struct RaftTransportIncomingBuilder<
    P: Clone + 'static,
    S: SharedState<InMemoryRaftTransportState<P>> + 'static,
> {
    transport: Option<Box<dyn TransportIncomingAdapter<RaftMessage<P>>>>,
    shared_state: Option<S>,
    peer_id: Option<PeerId>,
}

impl<P: Clone + 'static, S: SharedState<InMemoryRaftTransportState<P>> + 'static>
    RaftTransportIncomingBuilder<P, S>
{
    pub fn new() -> Self {
        Self {
            transport: None,
            shared_state: None,
            peer_id: None,
        }
    }

    pub fn with_shared_state(mut self, shared_state: S) -> Self {
        self.shared_state = Some(shared_state);
        self
    }

    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_variant(mut self, variant: RaftTransportIncomingVariant<P>) -> Self {
        match variant {
            RaftTransportIncomingVariant::InMemory => {
                // Requires peer_id and shared_state
                let peer_id = self
                    .peer_id
                    .expect("PeerId required for InMemory transport");
                let state = self
                    .shared_state
                    .clone()
                    .expect("SharedState required for InMemory transport");
                self.transport = Some(Box::new(InMemoryRaftTransport::new(peer_id, state)));
            }
            RaftTransportIncomingVariant::NoOp => {
                self.transport = Some(Box::new(NoOpRaftTransport::new()));
            }
            RaftTransportIncomingVariant::Custom(custom) => {
                self.transport = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn TransportIncomingAdapter<RaftMessage<P>>>, BuildError> {
        self.transport
            .ok_or(BuildError::MissingComponent("transport_incoming"))
    }
}
