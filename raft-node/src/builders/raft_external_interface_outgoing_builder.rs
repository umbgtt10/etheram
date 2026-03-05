// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::executor::outgoing::external_interface::client_response::RaftClientResponse;
use crate::variants::RaftExternalInterfaceOutgoingVariant;
use alloc::boxed::Box;
use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;

pub struct RaftExternalInterfaceOutgoingBuilder {
    ei: Option<Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>>,
}

impl RaftExternalInterfaceOutgoingBuilder {
    pub fn new() -> Self {
        Self { ei: None }
    }

    pub fn with_ei(
        mut self,
        ei: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>,
    ) -> Self {
        self.ei = Some(ei);
        self
    }

    pub fn with_variant(mut self, variant: RaftExternalInterfaceOutgoingVariant) -> Self {
        match variant {
            RaftExternalInterfaceOutgoingVariant::Custom(custom) => {
                self.ei = Some(custom);
            }
            RaftExternalInterfaceOutgoingVariant::InMemory => {
                panic!("InMemory EI requires SharedState — use RaftNodeBuilder or supply a pre-built InMemoryRaftExternalInterface via with_ei()");
            }
        }
        self
    }

    pub fn build(
        self,
    ) -> Result<Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>, BuildError> {
        self.ei.ok_or(BuildError::MissingComponent("ei_outgoing"))
    }
}

impl Default for RaftExternalInterfaceOutgoingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
