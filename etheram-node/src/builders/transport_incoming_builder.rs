// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::common_types::transport_incoming_adapter::TransportIncomingAdapter;
use crate::implementations::no_op_transport::NoOpTransport;
use crate::variants::IncomingTransportVariant;
use alloc::boxed::Box;

pub struct TransportIncomingBuilder {
    transport: Option<Box<dyn TransportIncomingAdapter<()>>>,
}

impl TransportIncomingBuilder {
    pub fn new() -> Self {
        Self { transport: None }
    }
    pub fn with_variant(mut self, variant: IncomingTransportVariant) -> Self {
        let transport = match variant {
            IncomingTransportVariant::NoOp => {
                Box::new(NoOpTransport) as Box<dyn TransportIncomingAdapter<()>>
            }
            IncomingTransportVariant::Custom(custom) => custom,
        };
        self.transport = Some(transport);
        self
    }
    pub fn build(self) -> Result<Box<dyn TransportIncomingAdapter<()>>, BuildError> {
        self.transport
            .ok_or(BuildError::MissingComponent("transport_incoming"))
    }
}

impl Default for TransportIncomingBuilder {
    fn default() -> Self {
        Self {
            transport: Some(Box::new(NoOpTransport)),
        }
    }
}
