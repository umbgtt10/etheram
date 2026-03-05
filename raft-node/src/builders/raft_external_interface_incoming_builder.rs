// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::incoming::external_interface::client_request::RaftClientRequest;
use crate::variants::RaftExternalInterfaceIncomingVariant;
use alloc::boxed::Box;
use etheram_core::node_common::build_error::BuildError;
use etheram_core::node_common::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;

pub struct RaftExternalInterfaceIncomingBuilder {
    ei: Option<Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>>,
}

impl RaftExternalInterfaceIncomingBuilder {
    pub fn new() -> Self {
        Self { ei: None }
    }

    pub fn with_ei(
        mut self,
        ei: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>,
    ) -> Self {
        self.ei = Some(ei);
        self
    }

    pub fn with_variant(mut self, variant: RaftExternalInterfaceIncomingVariant) -> Self {
        match variant {
            RaftExternalInterfaceIncomingVariant::Custom(custom) => {
                self.ei = Some(custom);
            }
            RaftExternalInterfaceIncomingVariant::InMemory => {
                panic!("InMemory EI requires SharedState — use RaftNodeBuilder or supply a pre-built InMemoryRaftExternalInterface via with_ei()");
            }
        }
        self
    }

    pub fn build(
        self,
    ) -> Result<Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>, BuildError> {
        self.ei.ok_or(BuildError::MissingComponent("ei_incoming"))
    }
}

impl Default for RaftExternalInterfaceIncomingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
