// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_transport::NoOpTransport;
use crate::variants::OutgoingTransportVariant;
use alloc::boxed::Box;
use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;

pub struct TransportOutgoingBuilder {
    transport: Option<Box<dyn TransportOutgoingAdapter<()>>>,
}

impl TransportOutgoingBuilder {
    pub fn new() -> Self {
        Self { transport: None }
    }
    pub fn with_variant(mut self, variant: OutgoingTransportVariant) -> Self {
        let transport = match variant {
            OutgoingTransportVariant::NoOp => {
                Box::new(NoOpTransport) as Box<dyn TransportOutgoingAdapter<()>>
            }
            OutgoingTransportVariant::Custom(custom) => custom,
        };
        self.transport = Some(transport);
        self
    }
    pub fn build(self) -> Result<Box<dyn TransportOutgoingAdapter<()>>, BuildError> {
        self.transport
            .ok_or(BuildError::MissingComponent("transport_outgoing"))
    }
}

impl Default for TransportOutgoingBuilder {
    fn default() -> Self {
        Self {
            transport: Some(Box::new(NoOpTransport)),
        }
    }
}
