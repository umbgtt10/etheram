// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_state::SyncState;

#[test]
fn highest_peer_height_without_observations_returns_none() {
    // Arrange
    let state = SyncState::new();

    // Act
    let highest = state.highest_peer_height();

    // Assert
    assert!(highest.is_none());
}

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

#[test]
fn complete_in_flight_request_non_matching_request_returns_false() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    let _ = state.next_request(10, 32);

    // Act
    let completed = state.complete_in_flight_request(3, 10);
    let second = state.next_request(10, 32);

    // Assert
    assert!(!completed);
    assert!(second.is_none());
}

#[test]
fn next_request_with_different_heights_picks_highest_height_peer() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 18);
    state.observe_status(3, 21);
    state.observe_status(4, 19);

    // Act
    let planned = state.next_request(10, 32);

    // Assert
    assert_eq!(planned, Some((3, 10, 32)));
}

#[test]
fn fail_in_flight_request_matching_request_switches_to_next_peer() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    state.observe_status(3, 20);
    let planned = state.next_request(10, 32);

    // Act
    let failed = state.fail_in_flight_request(3, 10);
    let planned_again = state.next_request(10, 32);

    // Assert
    assert_eq!(planned, Some((3, 10, 32)));
    assert!(failed);
    assert_eq!(planned_again, Some((2, 10, 32)));
}

#[test]
fn fail_in_flight_request_non_matching_request_returns_false() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    let _ = state.next_request(10, 32);

    // Act
    let failed = state.fail_in_flight_request(3, 10);
    let second = state.next_request(10, 32);

    // Assert
    assert!(!failed);
    assert!(second.is_none());
}

#[test]
fn next_request_when_local_at_tip_returns_none() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 10);
    state.observe_status(3, 10);

    // Act
    let planned = state.next_request(10, 32);

    // Assert
    assert!(planned.is_none());
}

#[test]
fn fail_in_flight_request_when_all_peers_failed_for_height_returns_none() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    state.observe_status(3, 20);
    let first = state.next_request(10, 32).expect("expected first request");
    let first_failed = state.fail_in_flight_request(first.0, first.1);
    let second = state.next_request(10, 32).expect("expected second request");

    // Act
    let second_failed = state.fail_in_flight_request(second.0, second.1);
    let planned_again = state.next_request(10, 32);

    // Assert
    assert!(first_failed);
    assert!(second_failed);
    assert!(planned_again.is_none());
}

#[test]
fn complete_in_flight_request_after_failover_clears_failed_peer_filter() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    state.observe_status(3, 20);
    let first = state.next_request(10, 32).expect("expected first request");
    let first_failed = state.fail_in_flight_request(first.0, first.1);
    let second = state.next_request(10, 32).expect("expected second request");

    // Act
    let completed = state.complete_in_flight_request(second.0, second.1);
    let planned_again = state.next_request(10, 32);

    // Assert
    assert!(first_failed);
    assert!(completed);
    assert_eq!(planned_again, Some((3, 10, 32)));
}

#[test]
fn next_request_after_successful_import_uses_updated_local_height() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    let first = state.next_request(10, 64).expect("expected first request");

    // Act
    let completed = state.complete_in_flight_request(first.0, first.1);
    let second = state.next_request(15, 64);

    // Assert
    assert!(completed);
    assert_eq!(second, Some((2, 15, 64)));
}
