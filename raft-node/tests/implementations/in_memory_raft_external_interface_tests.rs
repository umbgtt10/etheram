// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::node_common::shared_state::StdSharedState;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterface;
use raft_node::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterfaceState;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;

#[test]
fn poll_request_empty_queue_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftExternalInterfaceState::new());
    let ei = InMemoryRaftExternalInterface::new(1, state);

    // Act
    let request = ei.poll_request();

    // Assert
    assert!(request.is_none());
}

#[test]
fn push_request_then_poll_request_returns_client_and_payload() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftExternalInterfaceState::new());
    state.with_mut(|s| s.push_request(1, 9, RaftClientRequest::Command(vec![1, 2, 3])));
    let ei = InMemoryRaftExternalInterface::new(1, state);

    // Act
    let request = ei.poll_request();

    // Assert
    assert!(matches!(request, Some((9, RaftClientRequest::Command(_)))));
}

#[test]
fn send_response_then_drain_responses_returns_response() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftExternalInterfaceState::new());
    let ei = InMemoryRaftExternalInterface::new(1, state.clone());

    // Act
    ei.send_response(7, RaftClientResponse::Timeout);
    let responses = state.with_mut(|s| s.drain_responses(1));

    // Assert
    assert_eq!(responses.len(), 1);
    assert!(matches!(responses[0], (7, RaftClientResponse::Timeout)));
}

#[test]
fn send_response_then_drain_client_responses_returns_response() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftExternalInterfaceState::new());
    let ei = InMemoryRaftExternalInterface::new(1, state.clone());

    // Act
    ei.send_response(11, RaftClientResponse::Timeout);
    let responses = state.with_mut(|s| s.drain_client_responses(11));

    // Assert
    assert_eq!(responses.len(), 1);
    assert!(matches!(responses[0], RaftClientResponse::Timeout));
}
