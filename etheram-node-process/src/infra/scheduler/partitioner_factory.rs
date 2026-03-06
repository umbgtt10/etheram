// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::type_based_partitioner::TypeBasedPartitioner;
use etheram_node::partitioner::partition::Partitioner;

pub fn build_partitioner() -> Result<Box<dyn Partitioner<IbftMessage>>, String> {
    Ok(Box::new(TypeBasedPartitioner::new()))
}
