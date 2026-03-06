// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::partitioner_builder::PartitionerBuilder;
use etheram_node::partitioner::partition::Partitioner;

pub fn build_partitioner() -> Result<Box<dyn Partitioner<()>>, String> {
    PartitionerBuilder::default()
        .build()
        .map_err(|error| format!("failed to build partitioner: {error:?}"))
}
