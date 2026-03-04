// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_raft_transport::NoOpRaftTransport;
use crate::variants::RaftTransportIncomingVariant;
use alloc::boxed::Box;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::transport_incoming_adapter::TransportIncomingAdapter;

pub struct RaftTransportIncomingBuilder<P: Clone + 'static> {
    transport: Option<Box<dyn TransportIncomingAdapter<RaftMessage<P>>>>,
}

impl<P: Clone + 'static> RaftTransportIncomingBuilder<P> {
    pub fn new() -> Self {
        Self { transport: None }
    }

    pub fn with_transport(
        mut self,
        transport: Box<dyn TransportIncomingAdapter<RaftMessage<P>>>,
    ) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn with_variant(mut self, variant: RaftTransportIncomingVariant<P>) -> Self {
        match variant {
            RaftTransportIncomingVariant::NoOp => {
                self.transport = Some(Box::new(NoOpRaftTransport::<P>::new()));
            }
            RaftTransportIncomingVariant::Custom(custom) => {
                self.transport = Some(custom);
            }
            RaftTransportIncomingVariant::InMemory => {
                panic!("InMemory transport requires SharedState — use RaftNodeBuilder or supply a pre-built InMemoryRaftTransport via with_transport()");
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn TransportIncomingAdapter<RaftMessage<P>>>, BuildError> {
        self.transport
            .ok_or(BuildError::MissingComponent("transport_incoming"))
    }
}

impl<P: Clone + 'static> Default for RaftTransportIncomingBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}
