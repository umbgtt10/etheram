// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use crate::builders::error::BuildError;
use crate::implementations::no_op_raft_transport::NoOpRaftTransport;
use crate::variants::RaftTransportOutgoingVariant;
use alloc::boxed::Box;
use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;

pub struct RaftTransportOutgoingBuilder<P: Clone + 'static> {
    transport: Option<Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>>,
}

impl<P: Clone + 'static> RaftTransportOutgoingBuilder<P> {
    pub fn new() -> Self {
        Self { transport: None }
    }

    pub fn with_transport(
        mut self,
        transport: Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>,
    ) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn with_variant(mut self, variant: RaftTransportOutgoingVariant<P>) -> Self {
        match variant {
            RaftTransportOutgoingVariant::NoOp => {
                self.transport = Some(Box::new(NoOpRaftTransport::<P>::new()));
            }
            RaftTransportOutgoingVariant::Custom(custom) => {
                self.transport = Some(custom);
            }
            RaftTransportOutgoingVariant::InMemory => {
                panic!("InMemory transport requires SharedState — use RaftNodeBuilder or supply a pre-built InMemoryRaftTransport via with_transport()");
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>, BuildError> {
        self.transport
            .ok_or(BuildError::MissingComponent("transport_outgoing"))
    }
}

impl<P: Clone + 'static> Default for RaftTransportOutgoingBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}
