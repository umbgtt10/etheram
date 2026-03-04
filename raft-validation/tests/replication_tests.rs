// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::make_kv_command;
use crate::common::raft_cluster_helpers::replicate_one;
use crate::common::raft_cluster_helpers::setup_elected_3_node_cluster;
use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;

#[test]
fn submit_command_to_leader_increases_log_length() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let before = cluster.node_log_length(leader_idx);

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));
    cluster.drain_all();

    // Assert
    assert!(cluster.node_log_length(leader_idx) > before);
}

#[test]
fn single_command_replicated_to_all_followers_in_3_node_cluster() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let leader_log_before = cluster.node_log_length(leader_idx);

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));

    // Assert
    for i in 0..cluster.node_count() {
        assert!(cluster.node_log_length(i) > leader_log_before);
    }
}

#[test]
fn commit_index_advances_after_majority_ack() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));

    // Assert
    assert!(cluster.node_commit_index(leader_idx) > 0);
}

#[test]
fn all_nodes_have_same_commit_index_after_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));
    let expected = cluster.node_commit_index(leader_idx);

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_commit_index(i), expected);
    }
}

#[test]
fn all_nodes_have_same_log_length_after_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));
    let expected = cluster.node_log_length(leader_idx);

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_log_length(i), expected);
    }
}

#[test]
fn multiple_commands_increment_commit_index_in_order() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    for n in 1u8..=3 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
    }

    // Assert
    assert_eq!(cluster.node_commit_index(leader_idx), 3);
}

#[test]
fn log_length_equals_command_count_after_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    for n in 1u8..=5 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
    }

    // Assert
    assert_eq!(cluster.node_log_length(leader_idx), 5);
}

#[test]
fn replication_works_in_5_node_cluster() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("x", b"1"));

    // Assert
    let commit = cluster.node_commit_index(leader_idx);
    assert_eq!(commit, 1);
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_commit_index(i), commit);
    }
}

#[test]
fn last_applied_matches_commit_index_after_drain() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("k", b"v"));

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_last_applied(i), cluster.node_commit_index(i));
    }
}

#[test]
fn heartbeat_keeps_followers_in_follower_role() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    cluster.fire_timer(
        leader_idx,
        raft_node::incoming::timer::timer_event::RaftTimerEvent::Heartbeat,
    );
    cluster.drain_all();

    // Assert
    for i in 0..cluster.node_count() {
        if i != leader_idx {
            assert_eq!(
                cluster.node_role(i),
                raft_node::common_types::node_role::NodeRole::Follower
            );
        }
    }
}

#[test]
fn three_commands_all_nodes_log_length_equals_three() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();

    // Act
    for n in 0u8..3 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
    }

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_log_length(i), 3);
    }
}

#[test]
fn commit_index_monotonically_increases() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let mut prev = cluster.node_commit_index(leader_idx);

    // Act & Assert
    for n in 1u8..=4 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
        let current = cluster.node_commit_index(leader_idx);
        assert!(current > prev);
        prev = current;
    }
}
