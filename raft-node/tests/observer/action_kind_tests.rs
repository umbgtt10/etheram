// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::node_role::NodeRole;
use raft_node::common_types::snapshot::RaftSnapshot;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_node::observer::{action_kind, RaftActionKind};

#[test]
fn action_kind_set_term_action_returns_set_term_kind() {
    // Arrange
    let action = RaftAction::<()>::SetTerm(5);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SetTerm);
}

#[test]
fn action_kind_set_voted_for_action_returns_set_voted_for_kind() {
    // Arrange
    let action = RaftAction::<()>::SetVotedFor(Some(2));

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SetVotedFor);
}

#[test]
fn action_kind_append_entries_action_returns_append_entries_kind() {
    // Arrange
    let action = RaftAction::<()>::AppendEntries(vec![]);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::AppendEntries);
}

#[test]
fn action_kind_truncate_log_from_action_returns_truncate_log_from_kind() {
    // Arrange
    let action = RaftAction::<()>::TruncateLogFrom(3);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::TruncateLogFrom);
}

#[test]
fn action_kind_save_snapshot_action_returns_save_snapshot_kind() {
    // Arrange
    let action = RaftAction::<()>::SaveSnapshot(RaftSnapshot {
        last_included_index: 10,
        last_included_term: 2,
        data: vec![],
    });

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SaveSnapshot);
}

#[test]
fn action_kind_advance_commit_index_action_returns_advance_commit_index_kind() {
    // Arrange
    let action = RaftAction::<()>::AdvanceCommitIndex(7);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::AdvanceCommitIndex);
}

#[test]
fn action_kind_transition_role_action_returns_transition_role_kind() {
    // Arrange
    let action = RaftAction::<()>::TransitionRole(NodeRole::Leader);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::TransitionRole);
}

#[test]
fn action_kind_set_leader_id_action_returns_set_leader_id_kind() {
    // Arrange
    let action = RaftAction::<()>::SetLeaderId(Some(1));

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SetLeaderId);
}

#[test]
fn action_kind_update_match_index_action_preserves_peer_id() {
    // Arrange
    let action = RaftAction::<()>::UpdateMatchIndex {
        peer_id: 42,
        index: 5,
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::UpdateMatchIndex { peer_id: 42 });
}

#[test]
fn action_kind_update_next_index_action_preserves_peer_id() {
    // Arrange
    let action = RaftAction::<()>::UpdateNextIndex {
        peer_id: 7,
        index: 3,
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::UpdateNextIndex { peer_id: 7 });
}

#[test]
fn action_kind_send_message_action_preserves_recipient_peer_id() {
    // Arrange
    let action = RaftAction::<()>::SendMessage {
        to: 3,
        message: RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SendMessage { to: 3 });
}

#[test]
fn action_kind_broadcast_message_action_returns_broadcast_message_kind() {
    // Arrange
    let action = RaftAction::<()>::BroadcastMessage {
        message: RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::BroadcastMessage);
}

#[test]
fn action_kind_schedule_timeout_action_returns_schedule_timeout_kind() {
    // Arrange
    let action = RaftAction::<()>::ScheduleTimeout {
        event: RaftTimerEvent::ElectionTimeout,
        delay: 150,
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::ScheduleTimeout);
}

#[test]
fn action_kind_apply_to_state_machine_action_returns_apply_to_state_machine_kind() {
    // Arrange
    let action = RaftAction::<()>::ApplyToStateMachine {
        client_id: None,
        index: 1,
        payload_bytes: vec![],
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::ApplyToStateMachine);
}

#[test]
fn action_kind_send_client_response_action_preserves_client_id() {
    // Arrange
    let action = RaftAction::<()>::SendClientResponse {
        client_id: 99,
        response: RaftClientResponse::Timeout,
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::SendClientResponse { client_id: 99 });
}

#[test]
fn action_kind_log_action_returns_log_kind() {
    // Arrange
    let action = RaftAction::<()>::Log("consensus reached".to_string());

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::Log);
}

#[test]
fn action_kind_restore_from_snapshot_action_returns_restore_from_snapshot_kind() {
    // Arrange
    let action = RaftAction::<()>::RestoreFromSnapshot(alloc::vec![1u8, 2, 3]);

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::RestoreFromSnapshot);
}

#[test]
fn action_kind_query_state_machine_action_returns_query_state_machine_kind() {
    // Arrange
    let action = RaftAction::<()>::QueryStateMachine {
        client_id: 7,
        key: "k".to_string(),
    };

    // Act
    let kind = action_kind(&action);

    // Assert
    assert_eq!(kind, RaftActionKind::QueryStateMachine);
}
