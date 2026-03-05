// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::partitioner::partition::RaftPartitioner;
use etheram_core::collection::Collection;
use etheram_core::node_common::action_collection::ActionCollection;

pub struct TypeBasedRaftPartitioner;

impl TypeBasedRaftPartitioner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeBasedRaftPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Clone + 'static> RaftPartitioner<P> for TypeBasedRaftPartitioner {
    fn partition(
        &self,
        actions: &ActionCollection<RaftAction<P>>,
    ) -> (
        ActionCollection<RaftAction<P>>,
        ActionCollection<RaftAction<P>>,
    ) {
        let mut mutations = ActionCollection::new();
        let mut outputs = ActionCollection::new();
        for action in actions.iter() {
            match action {
                RaftAction::SetTerm(_) => mutations.push(action.clone()),
                RaftAction::SetVotedFor(_) => mutations.push(action.clone()),
                RaftAction::AppendEntries(_) => mutations.push(action.clone()),
                RaftAction::TruncateLogFrom(_) => mutations.push(action.clone()),
                RaftAction::SaveSnapshot(_) => mutations.push(action.clone()),
                RaftAction::AdvanceCommitIndex(_) => mutations.push(action.clone()),
                RaftAction::TransitionRole(_) => mutations.push(action.clone()),
                RaftAction::SetLeaderId(_) => mutations.push(action.clone()),
                RaftAction::UpdateMatchIndex { .. } => mutations.push(action.clone()),
                RaftAction::UpdateNextIndex { .. } => mutations.push(action.clone()),
                RaftAction::SendMessage { .. } => outputs.push(action.clone()),
                RaftAction::BroadcastMessage { .. } => outputs.push(action.clone()),
                RaftAction::ScheduleTimeout { .. } => outputs.push(action.clone()),
                RaftAction::ApplyToStateMachine { .. } => outputs.push(action.clone()),
                RaftAction::QueryStateMachine { .. } => outputs.push(action.clone()),
                RaftAction::RestoreFromSnapshot(_) => outputs.push(action.clone()),
                RaftAction::SendClientResponse { .. } => outputs.push(action.clone()),
                RaftAction::Log(_) => outputs.push(action.clone()),
            }
        }
        (mutations, outputs)
    }
}
