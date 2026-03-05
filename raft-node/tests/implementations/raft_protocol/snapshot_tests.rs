// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::implementations::raft::snapshot;

use crate::common::test_context::make_ctx_with_term;

#[test]
fn handle_install_snapshot_stale_term_returns_failure() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 5);

    // Act
    let actions = snapshot::handle_install_snapshot(&mut protocol, &ctx, 2, 3, 2, 1, 1, vec![]);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::InstallSnapshotResponse { success: false, .. },
            ..
        }
    )));
}

#[test]
fn handle_install_snapshot_already_committed_returns_success_without_install() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let mut ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 1);
    ctx.commit_index = 10;

    // Act
    let actions = snapshot::handle_install_snapshot(&mut protocol, &ctx, 2, 1, 2, 5, 1, vec![]);

    // Assert
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::InstallSnapshotResponse { success: true, .. },
            ..
        }
    )));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, RaftAction::SaveSnapshot(_))));
}

#[test]
fn handle_install_snapshot_valid_installs_snapshot_and_restores() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Follower, 1);

    // Act
    let actions = snapshot::handle_install_snapshot(&mut protocol, &ctx, 2, 1, 2, 5, 1, vec![9u8]);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::SaveSnapshot(_))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::RestoreFromSnapshot(_))));
    assert!(actions.iter().any(|a| matches!(
        a,
        RaftAction::SendMessage {
            message: RaftMessage::InstallSnapshotResponse { success: true, .. },
            ..
        }
    )));
}

#[test]
fn handle_install_snapshot_higher_term_steps_down_first() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let mut ctx = make_ctx_with_term(1, vec![2, 3], NodeRole::Leader, 1);
    ctx.leader_id = Some(1);

    // Act
    let actions = snapshot::handle_install_snapshot(&mut protocol, &ctx, 2, 5, 2, 5, 1, vec![]);

    // Assert
    assert!(actions.iter().any(|a| matches!(a, RaftAction::SetTerm(5))));
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::TransitionRole(NodeRole::Follower))));
}
