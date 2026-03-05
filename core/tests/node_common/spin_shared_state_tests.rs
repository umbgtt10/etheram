// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::shared_state::SharedState;
use etheram_core::node_common::spin_shared_state::SpinSharedState;

#[test]
fn with_mut_then_with_returns_updated_value() {
    // Arrange
    let state = SpinSharedState::new(10u64);

    // Act
    state.with_mut(|value| *value = 42);
    let current = state.with(|value| *value);

    // Assert
    assert_eq!(current, 42);
}

#[test]
fn clone_shares_same_underlying_state() {
    // Arrange
    let state = SpinSharedState::new(1u64);
    let cloned = state.clone();

    // Act
    cloned.with_mut(|value| *value = 99);
    let current = state.with(|value| *value);

    // Assert
    assert_eq!(current, 99);
}
