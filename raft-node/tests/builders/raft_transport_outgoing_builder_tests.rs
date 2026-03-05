// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::builders::raft_transport_outgoing_builder::RaftTransportOutgoingBuilder;
use raft_node::variants::RaftTransportOutgoingVariant;

#[test]
fn build_without_transport_returns_error() {
    // Arrange
    let builder = RaftTransportOutgoingBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_noop_builds_successfully() {
    // Arrange
    let builder = RaftTransportOutgoingBuilder::<Vec<u8>>::new();

    // Act
    let result = builder
        .with_variant(RaftTransportOutgoingVariant::NoOp)
        .build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_build_returns_error() {
    // Arrange
    let builder = RaftTransportOutgoingBuilder::<Vec<u8>>::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}
