// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_transport::NoOpTransport;
use crate::variants::IncomingTransportVariant;
use alloc::boxed::Box;
use etheram::common_types::transport_incoming_adapter::TransportInputAdapter;

pub struct TransportIncomingBuilder {
    transport: Option<Box<dyn TransportInputAdapter<()>>>,
}

impl TransportIncomingBuilder {
    pub fn new() -> Self {
        Self { transport: None }
    }
    pub fn with_variant(mut self, variant: IncomingTransportVariant) -> Self {
        let transport = match variant {
            IncomingTransportVariant::NoOp => {
                Box::new(NoOpTransport) as Box<dyn TransportInputAdapter<()>>
            }
            IncomingTransportVariant::Custom(custom) => custom,
        };
        self.transport = Some(transport);
        self
    }
    pub fn build(self) -> Result<Box<dyn TransportInputAdapter<()>>, BuildError> {
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
