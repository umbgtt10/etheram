// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::builders::raft_external_interface_incoming_builder::RaftExternalInterfaceIncomingBuilder;

#[test]
fn build_without_ei_returns_error() {
    // Arrange
    let builder = RaftExternalInterfaceIncomingBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn default_build_returns_error() {
    // Arrange
    let builder = RaftExternalInterfaceIncomingBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}
