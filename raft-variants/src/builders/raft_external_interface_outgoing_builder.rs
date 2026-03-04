// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterface;
use crate::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterfaceState;
use crate::implementations::shared_state::SharedState;
use crate::variants::RaftExternalInterfaceOutgoingVariant;
use alloc::boxed::Box;
use etheram_core::types::PeerId;
use raft_node::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;

pub struct RaftExternalInterfaceOutgoingBuilder<
    S: SharedState<InMemoryRaftExternalInterfaceState> + 'static,
> {
    ei: Option<Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>>,
    shared_state: Option<S>,
    peer_id: Option<PeerId>,
}

impl<S: SharedState<InMemoryRaftExternalInterfaceState> + 'static>
    RaftExternalInterfaceOutgoingBuilder<S>
{
    pub fn new() -> Self {
        Self {
            ei: None,
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

    pub fn with_variant(mut self, variant: RaftExternalInterfaceOutgoingVariant) -> Self {
        match variant {
            RaftExternalInterfaceOutgoingVariant::InMemory => {
                let peer_id = self.peer_id.expect("PeerId required for InMemory EI");
                let state = self
                    .shared_state
                    .clone()
                    .expect("SharedState required for InMemory EI");
                self.ei = Some(Box::new(InMemoryRaftExternalInterface::new(peer_id, state)));
            }
            RaftExternalInterfaceOutgoingVariant::Custom(custom) => {
                self.ei = Some(custom);
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
