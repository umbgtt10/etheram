// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::node_infra_slot::NodeInfraSlot;
use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::spawned_node::SpawnedNode;
use embassy_executor::Spawner;

pub struct InMemoryInfrastructure {
    slots: [Option<NodeInfraSlot>; MAX_NODES],
}

impl InMemoryInfrastructure {
    pub fn new(slots: [Option<NodeInfraSlot>; MAX_NODES]) -> Self {
        Self { slots }
    }

    pub fn create_node(
        &mut self,
        spawner: &Spawner,
        index: usize,
        cancel: &'static CancellationToken,
        node_cancel: &'static CancellationToken,
    ) -> SpawnedNode {
        SpawnedNode::new(
            spawner,
            index,
            self.slots[index].take().unwrap(),
            cancel,
            node_cancel,
        )
    }
}
