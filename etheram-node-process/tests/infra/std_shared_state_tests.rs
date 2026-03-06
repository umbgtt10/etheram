// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::shared_state::SharedState;
use etheram_node_process::infra::std_shared_state::StdSharedState;

#[test]
fn with_mut_then_with_updates_value_returns_updated_value() {
    // Arrange
    let state = StdSharedState::new(1u64);

    // Act
    state.with_mut(|value| {
        *value = 5;
    });
    let current = state.with(|value| *value);

    // Assert
    assert_eq!(current, 5);
}

#[test]
fn clone_with_mut_then_with_on_original_returns_updated_value() {
    // Arrange
    let state = StdSharedState::new(10u64);
    let clone = state.clone();

    // Act
    clone.with_mut(|value| {
        *value = 42;
    });
    let current = state.with(|value| *value);

    // Assert
    assert_eq!(current, 42);
}
