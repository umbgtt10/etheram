// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::make_ctx;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::common_types::node_role::NodeRole;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_variants::implementations::raft_protocol::client;
use raft_variants::implementations::raft_protocol::raft_protocol::RaftProtocol;

#[test]
fn handle_client_command_not_leader_returns_not_leader_response() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Follower);

    // Act
    let actions = client::handle_client_command(&mut protocol, &ctx, 42, vec![1u8]);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        RaftAction::SendClientResponse {
            client_id: 42,
            response: RaftClientResponse::NotLeader(_),
        }
    ));
}

#[test]
fn handle_client_command_leader_appends_log_entry() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Leader);

    // Act
    let actions = client::handle_client_command(&mut protocol, &ctx, 42, vec![9u8]);

    // Assert
    assert!(actions
        .iter()
        .any(|a| matches!(a, RaftAction::AppendEntries(_))));
}

#[test]
fn handle_client_command_leader_records_pending_entry() {
    // Arrange
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Leader);

    // Act
    client::handle_client_command(&mut protocol, &ctx, 99, vec![5u8]);

    // Assert
    assert!(protocol.pending_client_entries.contains_key(&1));
    assert_eq!(*protocol.pending_client_entries.get(&1).unwrap(), 99u64);
}

#[test]
fn handle_client_query_always_returns_not_leader() {
    // Arrange
    let ctx = make_ctx(1, vec![2, 3], NodeRole::Leader);

    // Act
    let actions = client::handle_client_query(&ctx, 77);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        RaftAction::SendClientResponse {
            client_id: 77,
            response: RaftClientResponse::NotLeader(_),
        }
    ));
}
