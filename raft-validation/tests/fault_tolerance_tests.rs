// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::make_kv_command;
use crate::common::raft_cluster_helpers::replicate_one;
use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_raft_validation::raft_cluster::RaftCluster;
use raft_variants::implementations::raft_protocol::common::quorum_size;

#[test]
fn one_node_crash_in_5_node_cluster_continues_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let crashed = (leader_idx + 1) % 5;

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));
    cluster.drain_except(&[crashed]);

    // Assert
    assert!(cluster.node_commit_index(leader_idx) > 0);
}

#[test]
fn two_nodes_crash_in_5_node_cluster_continues_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let crashed0 = (leader_idx + 1) % 5;
    let crashed1 = (leader_idx + 2) % 5;

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));
    cluster.drain_except(&[crashed0, crashed1]);

    // Assert
    assert!(cluster.node_commit_index(leader_idx) > 0);
}

#[test]
fn three_node_crash_in_5_node_cluster_stalls_replication() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let active0 = leader_idx;
    let others: Vec<usize> = (0..5).filter(|&i| i != active0).collect();
    let crashed = &others[0..3];
    let commit_before = cluster.node_commit_index(leader_idx);

    // Act
    cluster.submit_command(active0, 1, make_kv_command("k", b"v"));
    cluster.drain_except(crashed);

    // Assert
    assert_eq!(cluster.node_commit_index(leader_idx), commit_before);
}

#[test]
fn leader_crash_triggers_new_election() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let old_term = cluster.node_current_term(leader_idx);

    // Act
    let other: usize = (leader_idx + 1) % 5;
    cluster.fire_timer(other, RaftTimerEvent::ElectionTimeout);
    cluster.drain_except(&[leader_idx]);

    // Assert
    let new_leader = cluster.find_leader();
    assert!(
        new_leader.is_some_and(|idx| idx != leader_idx)
            || (0..cluster.node_count())
                .filter(|&i| i != leader_idx)
                .any(|i| cluster.node_current_term(i) > old_term)
    );
}

#[test]
fn new_leader_has_higher_term_after_old_leader_isolated() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let old_term = cluster.node_current_term(leader_idx);

    // Act
    let other = (leader_idx + 1) % 5;
    cluster.fire_timer(other, RaftTimerEvent::ElectionTimeout);
    cluster.drain_except(&[leader_idx]);

    // Assert
    let max_term = (0..cluster.node_count())
        .filter(|&i| i != leader_idx)
        .map(|i| cluster.node_current_term(i))
        .max()
        .unwrap_or(0);
    assert!(max_term > old_term);
}

#[test]
fn quorum_size_is_majority_plus_one_for_5_nodes() {
    // Arrange
    let peer_count = 4;

    // Act & Assert
    assert_eq!(quorum_size(peer_count), 3);
}

#[test]
fn concurrent_election_timeouts_then_cluster_recovers_single_leader() {
    // Arrange
    let mut cluster = RaftCluster::new(5);

    // Act
    cluster.fire_timer(0, RaftTimerEvent::ElectionTimeout);
    cluster.fire_timer(1, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();

    // Assert
    let leader_count = (0..cluster.node_count())
        .filter(|&i| cluster.node_role(i) == raft_node::common_types::node_role::NodeRole::Leader)
        .count();
    assert_eq!(leader_count, 1);
}

#[test]
fn commit_does_not_advance_without_majority() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let commit_before = cluster.node_commit_index(leader_idx);

    let others: Vec<usize> = (0..5).filter(|&i| i != leader_idx).collect();
    let excluding = &others[..3];

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));
    cluster.drain_except(excluding);

    // Assert
    assert_eq!(cluster.node_commit_index(leader_idx), commit_before);
}

#[test]
fn rejoin_after_lag_receives_missing_entries() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let lagging = (leader_idx + 1) % 5;

    // Act
    cluster.submit_command(leader_idx, 1, make_kv_command("k", b"v"));
    cluster.drain_except(&[lagging]);
    let leader_commit = cluster.node_commit_index(leader_idx);

    cluster.fire_timer(leader_idx, RaftTimerEvent::Heartbeat);
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_commit_index(lagging), leader_commit);
}

#[test]
fn partitioned_node_has_stale_log_while_isolated() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let isolated = (leader_idx + 1) % 5;
    let log_before = cluster.node_log_length(isolated);

    // Act
    for n in 0u8..3 {
        cluster.submit_command(leader_idx, 1, make_kv_command("k", &[n]));
        cluster.drain_except(&[isolated]);
    }

    // Assert
    assert_eq!(cluster.node_log_length(isolated), log_before);
}

#[test]
fn network_partition_heals_and_isolated_node_catches_up() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let isolated = (leader_idx + 1) % 5;

    replicate_one(&mut cluster, leader_idx, make_kv_command("k1", b"v1"));
    cluster.drain_except(&[isolated]);
    let leader_commit = cluster.node_commit_index(leader_idx);

    // Act
    cluster.fire_timer(leader_idx, RaftTimerEvent::Heartbeat);
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_commit_index(isolated), leader_commit);
}

#[test]
fn follower_timeout_in_healthy_cluster_does_not_block_progress() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();
    let follower_idx = (leader_idx + 1) % 5;

    // Act
    cluster.fire_timer(follower_idx, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();

    let active_leader = cluster.elect_leader();
    cluster.submit_command(active_leader, 77, make_kv_command("k", b"v"));
    cluster.drain_all();

    // Assert
    assert!(cluster.node_commit_index(active_leader) > 0);
}
