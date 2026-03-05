// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::builders::raft_observer_builder::RaftObserverBuilder;
use raft_node::variants::RaftObserverVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = RaftObserverBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_noop_builds_successfully() {
    // Arrange
    let builder = RaftObserverBuilder::new();

    // Act
    let result = builder.with_variant(RaftObserverVariant::NoOp).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = RaftObserverBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
