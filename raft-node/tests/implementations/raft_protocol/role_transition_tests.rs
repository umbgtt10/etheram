// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::make_ctx;
use crate::common::test_context::make_ctx_with_term;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::common;
use raft_node::implementations::raft::election;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::implementations::raft::replication;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

#[test]
fn become_leader_transitions_role_and_schedules_heartbeat() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Candidate);

    // Act
    let actions = election::become_leader(&mut protocol, &ctx);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Leader))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::Heartbeat,
            ..
        }
    )));
}

#[test]
fn become_leader_initializes_match_and_next_index_for_all_peers() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Candidate);

    // Act
    let actions = election::become_leader(&mut protocol, &ctx);

    // Assert
    let match_updates: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, RaftAction::UpdateMatchIndex { .. }))
        .collect();
    let next_updates: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, RaftAction::UpdateNextIndex { .. }))
        .collect();
    assert_eq!(match_updates.len(), 2);
    assert_eq!(next_updates.len(), 2);
}

#[test]
fn step_down_emits_follower_transition_and_clears_voted_for() {
    // Arrange & Act
    let actions = common::step_down::<Vec<u8>>(3);

    // Assert
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(3))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::SetVotedFor(None))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Follower))));
}

#[test]
fn handle_request_vote_response_higher_term_steps_down() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Candidate, 2);

    // Act
    let actions = election::handle_request_vote_response(&mut protocol, &ctx, 2, 7, false);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Follower))));
}

#[test]
fn handle_append_entries_from_candidate_transitions_to_follower() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Candidate, 1);

    // Act
    let actions = replication::handle_append_entries(&mut protocol, &ctx, 2, 1, 2, 0, 0, vec![], 0);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Follower))));
}
