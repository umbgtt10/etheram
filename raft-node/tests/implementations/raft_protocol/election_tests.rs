// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::{make_ctx, make_ctx_with_term};
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::election;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

#[test]
fn handle_election_timeout_leader_role_reschedules_election_timeout() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Leader);

    // Act
    let actions = election::handle_election_timeout(&mut protocol, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::ElectionTimeout,
            ..
        }
    ));
}

#[test]
fn handle_election_timeout_follower_with_peers_transitions_to_pre_candidate() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Follower);

    // Act
    let actions = election::handle_election_timeout(&mut protocol, &ctx);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::PreCandidate))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::BroadcastMessage {
            message: RaftMessage::PreVoteRequest { .. }
        }
    )));
}

#[test]
fn handle_election_timeout_follower_no_peers_starts_election_directly() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![], NodeRole::Follower);

    // Act
    let actions = election::handle_election_timeout(&mut protocol, &ctx);

    // Assert
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(2))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::TransitionRole(NodeRole::Candidate)
            | RaftAction::TransitionRole(NodeRole::Leader)
    )));
}

#[test]
fn start_election_emits_set_term_voted_for_candidate_and_broadcast() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 2);

    // Act
    let actions = election::start_election(&mut protocol, &ctx);

    // Assert
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(3))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::SetVotedFor(Some(1)))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Candidate))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::BroadcastMessage {
            message: RaftMessage::RequestVote { .. }
        }
    )));
}

#[test]
fn handle_pre_vote_response_quorum_reached_starts_election() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let mut ctx = make_ctx(1, vec![2, 3], NodeRole::PreCandidate);
    ctx.current_term = 1;
    protocol.pre_votes_received.insert(1);

    // Act
    let actions = election::handle_pre_vote_response(&mut protocol, &ctx, 2, true);

    // Assert
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(2))));
}

#[test]
fn handle_pre_vote_response_not_pre_candidate_returns_empty() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Follower);

    // Act
    let actions = election::handle_pre_vote_response(&mut protocol, &ctx, 2, true);

    // Assert
    assert!(actions.is_empty());
}

#[test]
fn handle_request_vote_stale_term_returns_false_response() {
    // Arrange
    let mut _p = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 5);

    // Act
    let actions = election::handle_request_vote(&ctx, 2, 3, 2, 0, 0);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::RequestVoteResponse {
                vote_granted: false,
                ..
            },
            ..
        }
    )));
}

#[test]
fn handle_request_vote_response_quorum_reached_becomes_leader() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let mut ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Candidate, 2);
    ctx.voted_for = Some(1);
    protocol.votes_received.insert(1);

    // Act
    let actions = election::handle_request_vote_response(&mut protocol, &ctx, 2, 2, true);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Leader))));
}
