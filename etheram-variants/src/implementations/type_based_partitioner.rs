// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram::brain::protocol::action::Action;
use etheram::collections::action_collection::ActionCollection;
use etheram::partitioner::partition::Partitioner;
use etheram_core::collection::Collection;

pub struct TypeBasedPartitioner;

#[allow(clippy::type_complexity)]
impl TypeBasedPartitioner {
    pub fn new() -> Self {
        Self
    }
    fn partition_inner<M: Clone, A>(
        &self,
        actions: &A,
    ) -> (
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
    )
    where
        A: Collection<Item = Action<M>>,
    {
        let mut mutations = ActionCollection::new();
        let mut outputs = ActionCollection::new();
        let mut executions = ActionCollection::new();
        for action in actions.iter() {
            match action {
                Action::UpdateAccount { .. } => mutations.push(action.clone()),
                Action::IncrementHeight => mutations.push(action.clone()),
                Action::StoreBlock { .. } => mutations.push(action.clone()),
                Action::ExecuteBlock { .. } => executions.push(action.clone()),
                Action::UpdateCache { .. } => mutations.push(action.clone()),
                Action::SendMessage { .. } => outputs.push(action.clone()),
                Action::BroadcastMessage { .. } => outputs.push(action.clone()),
                Action::SendClientResponse { .. } => outputs.push(action.clone()),
                Action::ScheduleTimeout { .. } => outputs.push(action.clone()),
                Action::Log { .. } => outputs.push(action.clone()),
            }
        }
        (mutations, outputs, executions)
    }
}

impl Default for TypeBasedPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Clone + 'static> Partitioner<M> for TypeBasedPartitioner {
    fn partition(
        &self,
        actions: &ActionCollection<Action<M>>,
    ) -> (
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
    ) {
        self.partition_inner(actions)
    }
}
