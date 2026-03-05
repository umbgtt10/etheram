// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::std_shared_state::StdSharedState;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_node::implementations::in_memory_transport::{
    InMemoryTransport, InMemoryTransportState,
};

#[test]
fn poll_empty_inbox_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<()>::new());
    let transport = InMemoryTransport::new(0, state.clone());

    // Act & Assert
    assert!(transport.poll().is_none());
}

#[test]
fn push_message_then_poll_returns_message_with_sender() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<()>::new());
    let transport = InMemoryTransport::new(0, state.clone());
    state.with_mut(|state| {
        state.push_message(0, 1, ());
    });

    // Act
    let result = transport.poll();

    // Assert
    assert!(matches!(result, Some((1, ()))));
}

#[test]
fn send_then_poll_on_receiver_returns_message() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<()>::new());
    let sender = InMemoryTransport::new(0, state.clone());
    let receiver = InMemoryTransport::new(1, state.clone());

    // Act
    sender.send(1, ());

    // Assert
    let result = receiver.poll();
    assert!(matches!(result, Some((0, ()))));
}

#[test]
fn send_targets_correct_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<()>::new());
    let sender = InMemoryTransport::new(0, state.clone());
    let receiver_1 = InMemoryTransport::new(1, state.clone());
    let receiver_2 = InMemoryTransport::new(2, state.clone());

    // Act
    sender.send(2, ());

    // Assert
    assert!(receiver_1.poll().is_none());
    assert!(receiver_2.poll().is_some());
}

#[test]
fn push_message_different_node_id_not_visible_to_other_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<()>::new());
    let node_0 = InMemoryTransport::new(0, state.clone());
    let _node_1 = InMemoryTransport::new(1, state.clone());
    state.with_mut(|state| {
        state.push_message(1, 0, ());
    });

    // Act
    let result = node_0.poll();

    // Assert
    assert!(result.is_none());
}
