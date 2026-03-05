// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::node_common::spin_shared_state::SpinSharedState;
use etheram_core::types::ClientId;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::implementations::in_memory_external_interface::{
    InMemoryExternalInterface, InMemoryExternalInterfaceState,
};
use etheram_node::incoming::external_interface::client_request::ClientRequest;

#[test]
fn poll_request_empty_queue_returns_none() {
    // Arrange
    let state = SpinSharedState::new(InMemoryExternalInterfaceState::new());
    let ei = InMemoryExternalInterface::new(0, state.clone());

    // Act & Assert
    assert!(ei.poll_request().is_none());
}

#[test]
fn push_request_then_poll_returns_request() {
    // Arrange
    let state = SpinSharedState::new(InMemoryExternalInterfaceState::new());
    let ei = InMemoryExternalInterface::new(0, state.clone());
    state.with_mut(|state| {
        state.push_request(0, 1, ClientRequest::GetHeight);
    });

    // Act
    let result = ei.poll_request();

    // Assert
    assert!(matches!(result, Some((1, ClientRequest::GetHeight))));
}

#[test]
fn send_response_then_drain_returns_response() {
    // Arrange
    let state = SpinSharedState::new(InMemoryExternalInterfaceState::new());
    let ei = InMemoryExternalInterface::new(0, state.clone());
    let client_id: ClientId = 42;

    // Act
    ei.send_response(client_id, ClientResponse::Height(7));

    // Assert
    let responses = state.with_mut(|state| state.drain_responses(client_id));
    assert_eq!(responses.len(), 1);
    assert!(matches!(responses[0], ClientResponse::Height(7)));
}

#[test]
fn drain_responses_unknown_client_returns_empty() {
    // Arrange
    let state = SpinSharedState::new(InMemoryExternalInterfaceState::new());

    // Act
    let responses = state.with_mut(|state| state.drain_responses(99));

    // Assert
    assert!(responses.is_empty());
}

#[test]
fn send_response_routes_to_correct_client() {
    // Arrange
    let state = SpinSharedState::new(InMemoryExternalInterfaceState::new());
    let ei = InMemoryExternalInterface::new(0, state.clone());

    // Act
    ei.send_response(1, ClientResponse::Height(1));
    ei.send_response(2, ClientResponse::Height(2));

    // Assert
    let for_client_1 = state.with_mut(|state| state.drain_responses(1));
    let for_client_2 = state.with_mut(|state| state.drain_responses(2));
    assert!(matches!(for_client_1[0], ClientResponse::Height(1)));
    assert!(matches!(for_client_2[0], ClientResponse::Height(2)));
}
