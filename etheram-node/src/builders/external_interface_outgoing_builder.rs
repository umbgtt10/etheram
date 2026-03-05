// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::executor::outgoing::external_interface::client_response::ClientResponse;
use crate::implementations::no_op_external_interface::NoOpExternalInterface;
use crate::variants::OutgoingExternalInterfaceVariant;
use alloc::boxed::Box;
use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;

pub struct ExternalInterfaceOutgoingBuilder {
    external_interface: Option<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>>,
}

impl ExternalInterfaceOutgoingBuilder {
    pub fn new() -> Self {
        Self {
            external_interface: None,
        }
    }

    pub fn with_variant(mut self, variant: OutgoingExternalInterfaceVariant) -> Self {
        let external_interface = match variant {
            OutgoingExternalInterfaceVariant::NoOp => Box::new(NoOpExternalInterface),
            OutgoingExternalInterfaceVariant::Custom(custom) => custom,
        };
        self.external_interface = Some(external_interface);
        self
    }

    pub fn build(
        self,
    ) -> Result<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>, BuildError> {
        self.external_interface
            .ok_or(BuildError::MissingComponent("external_interface_outgoing"))
    }
}

impl Default for ExternalInterfaceOutgoingBuilder {
    fn default() -> Self {
        Self {
            external_interface: Some(Box::new(NoOpExternalInterface)),
        }
    }
}
