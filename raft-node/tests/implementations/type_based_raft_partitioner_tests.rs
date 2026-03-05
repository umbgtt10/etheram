// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::collections::action_collection::ActionCollection;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::type_based_raft_partitioner::TypeBasedRaftPartitioner;
use raft_node::partitioner::partition::RaftPartitioner;

#[test]
fn partition_with_mixed_actions_splits_mutations_and_outputs() {
    // Arrange
    let partitioner = TypeBasedRaftPartitioner::new();
    let mut actions: ActionCollection<RaftAction<Vec<u8>>> = ActionCollection::new();
    actions.push(RaftAction::SetTerm(2));
    actions.push(RaftAction::TransitionRole(NodeRole::Leader));
    actions.push(RaftAction::SendMessage {
        to: 2,
        message: RaftMessage::RequestVote {
            term: 2,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    });
    actions.push(RaftAction::ApplyToStateMachine {
        client_id: None,
        index: 1,
        payload_bytes: vec![1u8],
    });

    // Act
    let (mutations, outputs) = partitioner.partition(&actions);

    // Assert
    assert_eq!(mutations.len(), 2);
    assert_eq!(outputs.len(), 2);
}
