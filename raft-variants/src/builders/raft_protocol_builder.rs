// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::raft_protocol::raft_protocol::RaftProtocol;
use crate::variants::RaftProtocolVariant;
use alloc::boxed::Box;
use alloc::vec::Vec;
use raft_node::brain::protocol::boxed_protocol::BoxedRaftProtocol;

pub struct RaftProtocolBuilder<P: Clone + From<Vec<u8>> + AsRef<[u8]> + 'static> {
    brain: Option<BoxedRaftProtocol<P>>,
}

impl<P: Clone + From<Vec<u8>> + AsRef<[u8]> + 'static> RaftProtocolBuilder<P> {
    pub fn new() -> Self {
        Self { brain: None }
    }

    pub fn with_variant(mut self, variant: RaftProtocolVariant<P>) -> Self {
        match variant {
            RaftProtocolVariant::Raft => {
                self.brain = Some(Box::new(RaftProtocol::new()));
            }
            RaftProtocolVariant::Custom(custom) => {
                self.brain = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<BoxedRaftProtocol<P>, BuildError> {
        self.brain.ok_or(BuildError::MissingComponent("protocol"))
    }
}

impl<P: Clone + From<Vec<u8>> + AsRef<[u8]> + 'static> Default for RaftProtocolBuilder<P> {
    fn default() -> Self {
        Self {
            brain: Some(Box::new(RaftProtocol::new())),
        }
    }
}
