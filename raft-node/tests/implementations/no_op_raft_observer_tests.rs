// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::no_op_raft_observer::NoOpRaftObserver;
use raft_node::observer::EventLevel;
use raft_node::observer::RaftActionKind;
use raft_node::observer::RaftObserver;

#[test]
fn min_level_returns_none() {
    // Arrange
    let observer = NoOpRaftObserver::new();

    // Act
    let level = observer.min_level();

    // Assert
    assert_eq!(level, EventLevel::None);
}

#[test]
fn observer_methods_are_noops() {
    // Arrange
    let mut observer = NoOpRaftObserver::new();

    // Act
    observer.node_started(1);
    observer.message_received(1, &MessageSource::Timer);
    observer.context_built(1, 2, NodeRole::Follower, 3);
    observer.action_emitted(1, &RaftActionKind::SetTerm);
    observer.mutation_applied(1, &RaftActionKind::SetTerm);
    observer.output_executed(1, &RaftActionKind::Log);
    observer.step_completed(1, true);

    // Assert
    assert_eq!(observer.min_level(), EventLevel::None);
}
