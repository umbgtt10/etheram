// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_state::SyncState;

#[test]
fn lag_distance_with_higher_peer_height_returns_distance() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 12);
    state.observe_status(3, 15);

    // Act
    let lag_distance = state.lag_distance(10);

    // Assert
    assert_eq!(lag_distance, Some(5));
}

#[test]
fn lag_distance_with_local_at_or_above_peers_returns_none() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 8);
    state.observe_status(3, 10);

    // Act
    let lag_distance = state.lag_distance(10);

    // Assert
    assert!(lag_distance.is_none());
}

#[test]
fn next_request_with_lag_returns_highest_peer_and_marks_in_flight() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 12);
    state.observe_status(3, 15);

    // Act
    let planned = state.next_request(10, 64);
    let second = state.next_request(10, 64);

    // Assert
    assert_eq!(planned, Some((3, 10, 64)));
    assert!(second.is_none());
}

#[test]
fn complete_in_flight_request_matching_request_clears_in_flight() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    let planned = state.next_request(10, 32);

    // Act
    let completed = state.complete_in_flight_request(2, 10);
    let planned_again = state.next_request(10, 32);

    // Assert
    assert_eq!(planned, Some((2, 10, 32)));
    assert!(completed);
    assert_eq!(planned_again, Some((2, 10, 32)));
}
