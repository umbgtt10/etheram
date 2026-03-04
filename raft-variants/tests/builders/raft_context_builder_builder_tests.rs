// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_variants::builders::raft_context_builder_builder::RaftContextBuilderBuilder;
use raft_variants::variants::RaftContextBuilderVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = RaftContextBuilderBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_eager_builds_successfully() {
    // Arrange
    let builder = RaftContextBuilderBuilder::<Vec<u8>>::new()
        .with_peer_id(1)
        .with_peers(vec![1, 2, 3]);

    // Act
    let result = builder
        .with_variant(RaftContextBuilderVariant::Eager)
        .build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_build_returns_error() {
    // Arrange
    let builder = RaftContextBuilderBuilder::<Vec<u8>>::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}
