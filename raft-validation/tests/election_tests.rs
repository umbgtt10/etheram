// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::setup_elected_3_node_cluster;
use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;
use raft_node::common_types::node_role::NodeRole;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_validation::raft_cluster::RaftCluster;

#[test]
fn new_cluster_all_nodes_start_as_follower() {
    // Arrange
    let cluster = RaftCluster::new(5);

    // Act & Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_role(i), NodeRole::Follower);
    }
}

#[test]
fn new_cluster_all_nodes_have_term_zero() {
    // Arrange
    let cluster = RaftCluster::new(3);

    // Act & Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_current_term(i), 0);
    }
}

#[test]
fn fire_election_timeout_starts_election_process() {
    // Arrange
    let mut cluster = RaftCluster::new(3);

    // Act
    cluster.fire_timer(0, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();

    // Assert
    assert!(cluster.node_current_term(0) > 0);
}

#[test]
fn single_leader_elected_in_3_node_cluster() {
    // Arrange & Act
    let (cluster, _) = setup_elected_3_node_cluster();

    // Assert
    let leader_count = (0..cluster.node_count())
        .filter(|&i| cluster.node_role(i) == NodeRole::Leader)
        .count();
    assert_eq!(leader_count, 1);
}

#[test]
fn single_leader_elected_in_5_node_cluster() {
    // Arrange & Act
    let (cluster, _) = setup_elected_5_node_cluster();

    // Assert
    let leader_count = (0..cluster.node_count())
        .filter(|&i| cluster.node_role(i) == NodeRole::Leader)
        .count();
    assert_eq!(leader_count, 1);
}

#[test]
fn elected_leader_has_term_greater_than_zero() {
    // Arrange & Act
    let (cluster, leader_idx) = setup_elected_3_node_cluster();

    // Assert
    assert!(cluster.node_current_term(leader_idx) > 0);
}

#[test]
fn followers_have_same_term_as_leader_after_election() {
    // Arrange & Act
    let (cluster, leader_idx) = setup_elected_5_node_cluster();
    let leader_term = cluster.node_current_term(leader_idx);

    // Assert
    for i in 0..cluster.node_count() {
        assert_eq!(cluster.node_current_term(i), leader_term);
    }
}

#[test]
fn followers_know_leader_id_after_election() {
    // Arrange & Act
    let (cluster, leader_idx) = setup_elected_5_node_cluster();
    let leader_peer_id = cluster.node_peer_id(leader_idx);

    // Assert
    for i in 0..cluster.node_count() {
        if i != leader_idx {
            assert_eq!(cluster.node_leader_id(i), Some(leader_peer_id));
        }
    }
}

#[test]
fn leader_leader_id_is_self() {
    // Arrange & Act
    let (cluster, leader_idx) = setup_elected_3_node_cluster();
    let leader_peer_id = cluster.node_peer_id(leader_idx);

    // Assert
    assert_eq!(cluster.node_leader_id(leader_idx), Some(leader_peer_id));
}

#[test]
fn election_result_is_deterministic_with_single_timeout() {
    // Arrange
    let mut cluster = RaftCluster::new(3);

    // Act
    cluster.fire_timer(0, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();

    // Assert
    assert!(cluster.find_leader().is_some());
}

#[test]
fn second_election_timeout_on_different_node_creates_higher_term() {
    // Arrange
    let (mut cluster, _) = setup_elected_3_node_cluster();
    let term_before = cluster.node_current_term(1);

    // Act
    cluster.fire_timer(1, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();
    cluster.fire_timer(1, RaftTimerEvent::ElectionTimeout);
    cluster.drain_all();

    // Assert
    assert!(cluster.node_current_term(1) > term_before);
}

#[test]
fn node_count_reported_correctly() {
    // Arrange
    let cluster = RaftCluster::new(5);

    // Act & Assert
    assert_eq!(cluster.node_count(), 5);
}

#[test]
fn elected_leader_index_returned_by_elect_leader() {
    // Arrange & Act
    let (cluster, leader_idx) = setup_elected_5_node_cluster();

    // Assert
    assert_eq!(cluster.node_role(leader_idx), NodeRole::Leader);
}
