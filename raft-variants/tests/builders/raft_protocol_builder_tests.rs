// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_variants::builders::raft_protocol_builder::RaftProtocolBuilder;
use raft_variants::variants::RaftProtocolVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = RaftProtocolBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_raft_builds_successfully() {
    // Arrange
    let builder = RaftProtocolBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.with_variant(RaftProtocolVariant::Raft).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = RaftProtocolBuilder::<Vec<u8>>::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
