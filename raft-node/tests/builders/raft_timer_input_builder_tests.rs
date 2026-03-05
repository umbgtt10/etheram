// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::builders::raft_timer_input_builder::RaftTimerInputBuilder;

#[test]
fn build_without_timer_returns_error() {
    // Arrange
    let builder = RaftTimerInputBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn default_build_returns_error() {
    // Arrange
    let builder = RaftTimerInputBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}
