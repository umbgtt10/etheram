// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::{make_ctx, make_ctx_with_term};
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::implementations::raft::replication;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

#[test]
fn handle_heartbeat_not_leader_returns_empty() {
    // Arrange
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Follower);

    // Act
    let actions = replication::handle_heartbeat(&ctx);

    // Assert
    assert!(actions.is_empty());
}

#[test]
fn handle_heartbeat_leader_sends_append_entries_to_all_peers() {
    // Arrange
    let mut ctx = make_ctx(1, vec![2, 3], NodeRole::Leader);
    ctx.next_index.insert(2, 1);
    ctx.next_index.insert(3, 1);

    // Act
    let actions = replication::handle_heartbeat(&ctx);

    // Assert
    let send_count = actions
        .iter()
        .filter(|a| {
            matches!(
                a,
                RaftAction::SendMessage {
                    message: RaftMessage::AppendEntries { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(send_count, 2);
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::Heartbeat,
            ..
        }
    )));
}

#[test]
fn handle_append_entries_stale_term_returns_failure_response() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 5);

    // Act
    let actions = replication::handle_append_entries(&mut protocol, &ctx, 2, 3, 2, 0, 0, vec![], 0);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::AppendEntriesResponse { success: false, .. },
            ..
        }
    )));
}

#[test]
fn handle_append_entries_empty_consistent_log_returns_success() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 1);

    // Act
    let actions = replication::handle_append_entries(&mut protocol, &ctx, 2, 1, 2, 0, 0, vec![], 0);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::AppendEntriesResponse { success: true, .. },
            ..
        }
    )));
}

#[test]
fn handle_append_entries_valid_entries_appended_to_log() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 1);
    let entries = vec![LogEntry {
        term: 1,
        index: 1,
        payload: vec![1u8],
    }];

    // Act
    let actions =
        replication::handle_append_entries(&mut protocol, &ctx, 2, 1, 2, 0, 0, entries, 0);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::AppendEntries(_))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::AppendEntriesResponse { success: true, .. },
            ..
        }
    )));
}

#[test]
fn handle_append_entries_response_success_advances_match_index() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let mut ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Leader, 1);
    ctx.match_index.insert(2, 0);
    ctx.next_index.insert(2, 2);

    // Act
    let actions = replication::handle_append_entries_response(&mut protocol, &ctx, 2, 1, true, 1);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::UpdateMatchIndex {
            peer_id: 2,
            index: 1
        }
    )));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::UpdateNextIndex {
            peer_id: 2,
            index: 2
        }
    )));
}

#[test]
fn handle_append_entries_response_higher_term_steps_down() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Leader, 1);

    // Act
    let actions = replication::handle_append_entries_response(&mut protocol, &ctx, 2, 5, false, 0);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Follower))));
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(5))));
}
