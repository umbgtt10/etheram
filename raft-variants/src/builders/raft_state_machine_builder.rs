// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::in_memory_raft_state_machine::InMemoryRaftStateMachine;
use crate::variants::RaftStateMachineVariant;
use alloc::boxed::Box;
use raft_node::common_types::state_machine::RaftStateMachine;

pub struct RaftStateMachineBuilder {
    sm: Option<Box<dyn RaftStateMachine>>,
}

impl RaftStateMachineBuilder {
    pub fn new() -> Self {
        Self { sm: None }
    }

    pub fn with_variant(mut self, variant: RaftStateMachineVariant) -> Self {
        match variant {
            RaftStateMachineVariant::InMemory => {
                self.sm = Some(Box::new(InMemoryRaftStateMachine::new()));
            }
            RaftStateMachineVariant::Custom(custom) => {
                self.sm = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn RaftStateMachine>, BuildError> {
        self.sm.ok_or(BuildError::MissingComponent("state_machine"))
    }
}

impl Default for RaftStateMachineBuilder {
    fn default() -> Self {
        Self {
            sm: Some(Box::new(InMemoryRaftStateMachine::new())),
        }
    }
}
