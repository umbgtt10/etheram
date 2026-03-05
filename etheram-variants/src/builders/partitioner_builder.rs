// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::type_based_partitioner::TypeBasedPartitioner;
use crate::variants::PartitionerVariant;
use alloc::boxed::Box;
use etheram_node::partitioner::partition::Partitioner;

pub struct PartitionerBuilder {
    partitioner: Option<Box<dyn Partitioner<()>>>,
}

impl PartitionerBuilder {
    pub fn new() -> Self {
        Self { partitioner: None }
    }
    pub fn with_variant(mut self, variant: PartitionerVariant) -> Self {
        let partitioner = match variant {
            PartitionerVariant::TypeBased => {
                Box::new(TypeBasedPartitioner::new()) as Box<dyn Partitioner<()>>
            }
            PartitionerVariant::Custom(custom) => custom,
        };
        self.partitioner = Some(partitioner);
        self
    }
    pub fn build(self) -> Result<Box<dyn Partitioner<()>>, BuildError> {
        self.partitioner
            .ok_or(BuildError::MissingComponent("partitioner"))
    }
}

impl Default for PartitionerBuilder {
    fn default() -> Self {
        Self::new().with_variant(PartitionerVariant::TypeBased)
    }
}
