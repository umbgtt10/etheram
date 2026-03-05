// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_node::implementations::no_op_transport::NoOpTransport;

#[test]
fn poll_always_returns_none() {
    // Arrange
    let transport = NoOpTransport;

    // Act & Assert
    assert!(transport.poll().is_none());
}

#[test]
fn send_does_not_panic() {
    // Arrange
    let transport = NoOpTransport;

    // Act & Assert
    transport.send(1, ());
}

#[test]
fn clone_produces_independent_instance() {
    // Arrange
    let transport = NoOpTransport;

    // Act
    let cloned = transport;

    // Assert
    assert!(cloned.poll().is_none());
}
