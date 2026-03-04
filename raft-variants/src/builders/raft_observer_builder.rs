// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_raft_observer::NoOpRaftObserver;
use crate::variants::RaftObserverVariant;
use alloc::boxed::Box;
use raft_node::observer::RaftObserver;

pub struct RaftObserverBuilder {
    observer: Option<Box<dyn RaftObserver>>,
}

impl RaftObserverBuilder {
    pub fn new() -> Self {
        Self { observer: None }
    }

    pub fn with_variant(mut self, variant: RaftObserverVariant) -> Self {
        match variant {
            RaftObserverVariant::NoOp => {
                self.observer = Some(Box::new(NoOpRaftObserver));
            }
            RaftObserverVariant::Custom(custom) => {
                self.observer = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn RaftObserver>, BuildError> {
        self.observer
            .ok_or(BuildError::MissingComponent("observer"))
    }
}

impl Default for RaftObserverBuilder {
    fn default() -> Self {
        Self {
            observer: Some(Box::new(NoOpRaftObserver)),
        }
    }
}
