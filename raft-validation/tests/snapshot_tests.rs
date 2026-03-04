// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::make_kv_command;
use crate::common::raft_cluster_helpers::replicate_one;
use crate::common::raft_cluster_helpers::setup_elected_3_node_cluster;
use raft_node::brain::protocol::message::RaftMessage;

#[test]
fn inject_install_snapshot_updates_commit_index() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let receiver = (leader_idx + 1) % 3;
    let leader_peer_id = cluster.node_peer_id(leader_idx);
    let leader_term = cluster.node_current_term(leader_idx);

    // Act
    cluster.inject_message(
        receiver,
        leader_peer_id,
        RaftMessage::InstallSnapshot {
            term: leader_term,
            leader_id: leader_peer_id,
            snapshot_index: 5,
            snapshot_term: leader_term,
            data: vec![0u8; 4],
        },
    );
    cluster.drain(receiver);

    // Assert
    assert_eq!(cluster.node_commit_index(receiver), 5);
}

#[test]
fn inject_install_snapshot_advances_last_applied() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let receiver = (leader_idx + 1) % 3;
    let leader_peer_id = cluster.node_peer_id(leader_idx);
    let leader_term = cluster.node_current_term(leader_idx);

    // Act
    cluster.inject_message(
        receiver,
        leader_peer_id,
        RaftMessage::InstallSnapshot {
            term: leader_term,
            leader_id: leader_peer_id,
            snapshot_index: 7,
            snapshot_term: leader_term,
            data: vec![],
        },
    );
    cluster.drain(receiver);

    // Assert
    assert_eq!(cluster.node_commit_index(receiver), 7);
}

#[test]
fn snapshot_with_higher_term_causes_step_down() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let receiver = (leader_idx + 1) % 3;
    let original_term = cluster.node_current_term(receiver);
    let sender = cluster.node_peer_id(leader_idx);

    // Act
    cluster.inject_message(
        receiver,
        sender,
        RaftMessage::InstallSnapshot {
            term: original_term + 10,
            leader_id: sender,
            snapshot_index: 3,
            snapshot_term: original_term + 10,
            data: vec![],
        },
    );
    cluster.drain(receiver);

    // Assert
    assert!(cluster.node_current_term(receiver) > original_term);
}

#[test]
fn snapshot_below_current_commit_is_ignored() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    replicate_one(&mut cluster, leader_idx, make_kv_command("k", b"v"));

    let receiver = (leader_idx + 1) % 3;
    let current_commit = cluster.node_commit_index(receiver);
    let leader_peer_id = cluster.node_peer_id(leader_idx);
    let leader_term = cluster.node_current_term(leader_idx);

    // Act
    cluster.inject_message(
        receiver,
        leader_peer_id,
        RaftMessage::InstallSnapshot {
            term: leader_term,
            leader_id: leader_peer_id,
            snapshot_index: current_commit.saturating_sub(1).max(0),
            snapshot_term: leader_term,
            data: vec![],
        },
    );
    cluster.drain(receiver);

    // Assert
    assert_eq!(cluster.node_commit_index(receiver), current_commit);
}

#[test]
fn fresh_node_can_receive_snapshot_and_advance_state() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let fresh_receiver = (leader_idx + 2) % 3;
    let leader_peer_id = cluster.node_peer_id(leader_idx);
    let leader_term = cluster.node_current_term(leader_idx);

    // Act
    cluster.inject_message(
        fresh_receiver,
        leader_peer_id,
        RaftMessage::InstallSnapshot {
            term: leader_term,
            leader_id: leader_peer_id,
            snapshot_index: 10,
            snapshot_term: leader_term,
            data: vec![0xAB, 0xCD],
        },
    );
    cluster.drain(fresh_receiver);

    // Assert
    assert_eq!(cluster.node_commit_index(fresh_receiver), 10);
}

#[test]
fn snapshot_response_accepted_by_leader() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let follower_peer_id = cluster.node_peer_id((leader_idx + 1) % 3);
    let leader_term = cluster.node_current_term(leader_idx);

    // Act
    cluster.inject_message(
        leader_idx,
        follower_peer_id,
        RaftMessage::InstallSnapshotResponse {
            term: leader_term,
            success: true,
        },
    );
    cluster.drain(leader_idx);

    // Assert
    assert_eq!(
        cluster.node_role(leader_idx),
        raft_node::common_types::node_role::NodeRole::Leader
    );
}
