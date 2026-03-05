// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::node_role::NodeRole;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_validation::raft_cluster::RaftCluster;

#[test]
fn fire_timer_all_heartbeat_keeps_single_leader_and_followers() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();

    // Act
    cluster.fire_timer_all(RaftTimerEvent::Heartbeat);
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_role(leader_idx), NodeRole::Leader);
    for i in 0..cluster.node_count() {
        if i != leader_idx {
            assert_eq!(cluster.node_role(i), NodeRole::Follower);
        }
    }
}

#[test]
fn inject_message_append_entries_updates_receiver_leader_id() {
    // Arrange
    let mut cluster = RaftCluster::new(3);
    let receiver_idx = 1;

    // Act
    cluster.inject_message(
        receiver_idx,
        1,
        RaftMessage::AppendEntries {
            term: 1,
            leader_id: 1,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::new(),
            leader_commit: 0,
        },
    );
    cluster.drain(receiver_idx);

    // Assert
    assert_eq!(cluster.node_leader_id(receiver_idx), Some(1));
    assert_eq!(cluster.node_current_term(receiver_idx), 1);
}
