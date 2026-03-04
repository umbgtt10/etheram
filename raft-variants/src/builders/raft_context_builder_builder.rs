// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::eager_raft_context_builder::EagerRaftContextBuilder;
use crate::variants::RaftContextBuilderVariant;
use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_node::context::context_builder::RaftContextBuilder;

pub struct RaftContextBuilderBuilder<P: Clone + 'static> {
    builder: Option<Box<dyn RaftContextBuilder<P>>>,
    peer_id: Option<PeerId>,
    peers: Vec<PeerId>,
}

impl<P: Clone + 'static> RaftContextBuilderBuilder<P> {
    pub fn new() -> Self {
        Self {
            builder: None,
            peer_id: None,
            peers: Vec::new(),
        }
    }

    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.peers = peers;
        self
    }

    pub fn with_variant(mut self, variant: RaftContextBuilderVariant<P>) -> Self {
        match variant {
            RaftContextBuilderVariant::Eager => {
                let peer_id = self
                    .peer_id
                    .expect("PeerId required for Eager context builder");
                let peers = self.peers.clone();
                self.builder = Some(Box::new(EagerRaftContextBuilder::new(peer_id, peers)));
            }
            RaftContextBuilderVariant::Custom(custom) => {
                self.builder = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn RaftContextBuilder<P>>, BuildError> {
        self.builder
            .ok_or(BuildError::MissingComponent("context_builder"))
    }
}
