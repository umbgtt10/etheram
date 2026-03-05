// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::implementations::no_op_external_interface::NoOpExternalInterface;

#[test]
fn poll_request_always_returns_none() {
    // Arrange
    let ei = NoOpExternalInterface;

    // Act & Assert
    assert!(ei.poll_request().is_none());
}

#[test]
fn send_response_does_not_panic() {
    // Arrange
    let ei = NoOpExternalInterface;
    let response = ClientResponse::Balance {
        balance: 0,
        height: 0,
    };

    // Act & Assert
    ei.send_response(1, response);
}

#[test]
fn clone_produces_independent_instance() {
    // Arrange
    let ei = NoOpExternalInterface;

    // Act
    let cloned = ei.clone();

    // Assert
    assert!(cloned.poll_request().is_none());
}
