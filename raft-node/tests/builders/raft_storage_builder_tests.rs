// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::builders::raft_storage_builder::RaftStorageBuilder;
use raft_node::variants::RaftStorageVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = RaftStorageBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_in_memory_builds_successfully() {
    // Arrange
    let builder = RaftStorageBuilder::<Vec<u8>>::new();

    // Act
    let result = builder.with_variant(RaftStorageVariant::InMemory).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = RaftStorageBuilder::<Vec<u8>>::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
