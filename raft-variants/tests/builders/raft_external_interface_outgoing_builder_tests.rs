// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_variants::builders::raft_external_interface_outgoing_builder::RaftExternalInterfaceOutgoingBuilder;

#[test]
fn build_without_ei_returns_error() {
    // Arrange
    let builder = RaftExternalInterfaceOutgoingBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn default_build_returns_error() {
    // Arrange
    let builder = RaftExternalInterfaceOutgoingBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}
