// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::type_based_raft_partitioner::TypeBasedRaftPartitioner;
use crate::partitioner::partition::RaftPartitioner;
use crate::variants::RaftPartitionerVariant;
use alloc::boxed::Box;
use etheram_core::node_common::build_error::BuildError;

pub struct RaftPartitionerBuilder<P: Clone + 'static> {
    partitioner: Option<Box<dyn RaftPartitioner<P>>>,
}

impl<P: Clone + 'static> RaftPartitionerBuilder<P> {
    pub fn new() -> Self {
        Self { partitioner: None }
    }

    pub fn with_variant(mut self, variant: RaftPartitionerVariant<P>) -> Self {
        match variant {
            RaftPartitionerVariant::TypeBased => {
                self.partitioner = Some(Box::new(TypeBasedRaftPartitioner::new()));
            }
            RaftPartitionerVariant::Custom(custom) => {
                self.partitioner = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn RaftPartitioner<P>>, BuildError> {
        self.partitioner
            .ok_or(BuildError::MissingComponent("partitioner"))
    }
}

impl<P: Clone + 'static> Default for RaftPartitionerBuilder<P> {
    fn default() -> Self {
        Self {
            partitioner: Some(Box::new(TypeBasedRaftPartitioner::new())),
        }
    }
}
