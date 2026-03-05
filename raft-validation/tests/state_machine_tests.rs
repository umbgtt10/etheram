// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::make_kv_command;
use crate::common::raft_cluster_helpers::replicate_one;
use crate::common::raft_cluster_helpers::setup_elected_3_node_cluster;
use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;
use raft_validation::raft_cluster::RaftCluster;

#[test]
fn applied_index_is_zero_before_any_commands() {
    // Arrange
    let cluster = RaftCluster::new(3);

    // Act & Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_last_applied(i), 0);
    }
}

#[test]
fn replicated_command_increments_last_applied_on_leader() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));

    // Assert
    assert!(cluster.node_last_applied(leader_idx) > 0);
}

#[test]
fn replicated_command_increments_last_applied_on_all_nodes() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("key", b"val"));

    // Assert
    for i in 0..cluster.node_count() {
        assert!(cluster.node_last_applied(i) > 0);
    }
}

#[test]
fn last_applied_equals_commit_index_after_drain() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    replicate_one(&mut cluster, leader_idx, make_kv_command("x", b"1"));

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_last_applied(i), cluster.node_commit_index(i));
    }
}

#[test]
fn multiple_commands_increment_last_applied_in_sequence() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    for n in 1u8..=3 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
    }

    // Assert
    assert_eq!(cluster.node_last_applied(leader_idx), 3);
}

#[test]
fn all_nodes_last_applied_consistent_after_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();

    // Act
    for n in 0u8..4 {
        replicate_one(&mut cluster, leader_idx, make_kv_command("k", &[n]));
    }
    let expected = cluster.node_last_applied(leader_idx);

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_last_applied(i), expected);
    }
}

#[test]
fn state_machine_not_applied_before_commit() {
    // Arrange
    let (cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));

    // Assert
    assert_eq!(cluster.node_last_applied(leader_idx), 0);
}

#[test]
fn last_applied_tracks_each_committed_entry() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act & Assert
    for expected_applied in 1u64..=5 {
        replicate_one(
            &mut cluster,
            leader_idx,
            make_kv_command("k", &[expected_applied as u8]),
        );
        assert_eq!(cluster.node_last_applied(leader_idx), expected_applied);
    }
}
