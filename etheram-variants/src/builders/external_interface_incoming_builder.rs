// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_external_interface::NoOpExternalInterface;
use crate::variants::IncomingExternalInterfaceVariant;
use alloc::boxed::Box;
use etheram_node::common_types::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

pub struct ExternalInterfaceIncomingBuilder {
    external_interface: Option<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>>,
}

impl ExternalInterfaceIncomingBuilder {
    pub fn new() -> Self {
        Self {
            external_interface: None,
        }
    }

    pub fn with_variant(mut self, variant: IncomingExternalInterfaceVariant) -> Self {
        let external_interface = match variant {
            IncomingExternalInterfaceVariant::NoOp => Box::new(NoOpExternalInterface),
            IncomingExternalInterfaceVariant::Custom(custom) => custom,
        };
        self.external_interface = Some(external_interface);
        self
    }

    pub fn build(
        self,
    ) -> Result<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>, BuildError> {
        self.external_interface
            .ok_or(BuildError::MissingComponent("external_interface_incoming"))
    }
}

impl Default for ExternalInterfaceIncomingBuilder {
    fn default() -> Self {
        Self {
            external_interface: Some(Box::new(NoOpExternalInterface)),
        }
    }
}
