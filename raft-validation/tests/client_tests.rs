// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::raft_cluster_helpers::make_kv_command;
use crate::common::raft_cluster_helpers::setup_elected_3_node_cluster;
use crate::common::raft_cluster_helpers::setup_elected_5_node_cluster;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;

#[test]
fn submit_command_to_leader_yields_applied_response() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    cluster.submit_command(leader_idx, 42, make_kv_command("k", b"v"));
    cluster.drain_all();
    let responses = cluster.drain_responses(leader_idx);

    // Assert
    assert!(responses
        .iter()
        .any(|(cid, resp)| { *cid == 42 && matches!(resp, RaftClientResponse::Applied(_)) }));
}

#[test]
fn submit_command_to_follower_yields_not_leader_response() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let follower_idx = (leader_idx + 1) % 3;

    // Act
    cluster.submit_command(follower_idx, 7, make_kv_command("k", b"v"));
    cluster.drain_all();
    let responses = cluster.drain_responses(follower_idx);

    // Assert
    assert!(responses
        .iter()
        .any(|(cid, resp)| { *cid == 7 && matches!(resp, RaftClientResponse::NotLeader(_)) }));
}

#[test]
fn no_response_before_drain_all() {
    // Arrange
    let (cluster, leader_idx) = setup_elected_3_node_cluster();

    // Act
    cluster.submit_command(leader_idx, 99, make_kv_command("k", b"v"));
    let responses = cluster.drain_responses(leader_idx);

    // Assert
    assert!(responses.is_empty());
}

#[test]
fn multiple_clients_each_receive_response() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_5_node_cluster();

    // Act
    for client_id in 1u64..=3 {
        cluster.submit_command(
            leader_idx,
            client_id,
            make_kv_command("k", &[client_id as u8]),
        );
        cluster.drain_all();
    }
    let responses = cluster.drain_responses(leader_idx);

    // Assert
    assert_eq!(responses.len(), 3);
    for client_id in 1u64..=3 {
        assert!(responses.iter().any(|(cid, _)| *cid == client_id));
    }
}

#[test]
fn follower_redirects_to_known_leader() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let follower_idx = (leader_idx + 1) % 3;
    let expected_leader_id = cluster.node_peer_id(leader_idx);

    // Act
    cluster.submit_command(follower_idx, 5, make_kv_command("k", b"v"));
    cluster.drain_all();
    let responses = cluster.drain_responses(follower_idx);

    // Assert
    assert!(responses.iter().any(|(_, resp)| {
        matches!(resp, RaftClientResponse::NotLeader(Some(id)) if *id == expected_leader_id)
    }));
}

#[test]
fn query_after_committed_write_on_leader_returns_query_result_value() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let write_client = 101;
    let read_client = 202;

    // Act
    cluster.submit_command(leader_idx, write_client, make_kv_command("k", b"v"));
    cluster.drain_all();
    cluster.fire_timer(
        leader_idx,
        raft_node::incoming::timer::timer_event::RaftTimerEvent::Heartbeat,
    );
    cluster.drain_all();
    cluster.submit_query(leader_idx, read_client, "k");
    cluster.drain_all();
    let responses = cluster.drain_client_responses(read_client);

    // Assert
    assert!(responses.iter().any(|resp| matches!(
        resp,
        RaftClientResponse::QueryResult(value) if value == b"v"
    )));
}

#[test]
fn query_response_can_be_drained_by_client_id() {
    // Arrange
    let (mut cluster, leader_idx) = setup_elected_3_node_cluster();
    let client_id = 303;

    // Act
    cluster.submit_query(leader_idx, client_id, "missing");
    cluster.drain_all();
    let responses = cluster.drain_client_responses(client_id);

    // Assert
    assert!(responses.iter().any(|resp| {
        matches!(
            resp,
            RaftClientResponse::QueryResult(value) if value.is_empty()
        )
    }));
}
