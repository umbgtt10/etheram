// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::implementations::ibft::vote_tracker::VoteTracker;

#[test]
fn has_quorum_empty_returns_false() {
    // Arrange
    let tracker = VoteTracker::new(3);

    // Act & Assert
    assert!(!tracker.has_quorum(0, 0, [0u8; 32]));
}

#[test]
fn record_below_quorum_returns_false() {
    // Arrange
    let mut tracker = VoteTracker::new(3);
    let hash = [1u8; 32];

    // Act
    tracker.record(0, 0, hash, 1);
    tracker.record(0, 0, hash, 2);

    // Assert
    assert!(!tracker.has_quorum(0, 0, hash));
}

#[test]
fn record_at_quorum_returns_true() {
    // Arrange
    let mut tracker = VoteTracker::new(3);
    let hash = [1u8; 32];

    // Act
    tracker.record(0, 0, hash, 1);
    tracker.record(0, 0, hash, 2);
    tracker.record(0, 0, hash, 3);

    // Assert
    assert!(tracker.has_quorum(0, 0, hash));
}

#[test]
fn record_duplicate_vote_does_not_double_count() {
    // Arrange
    let mut tracker = VoteTracker::new(2);
    let hash = [1u8; 32];

    // Act
    tracker.record(0, 0, hash, 1);
    tracker.record(0, 0, hash, 1);

    // Assert
    assert!(!tracker.has_quorum(0, 0, hash));
}

#[test]
fn has_quorum_different_block_hash_returns_false() {
    // Arrange
    let mut tracker = VoteTracker::new(2);
    let hash_a = [1u8; 32];
    let hash_b = [2u8; 32];

    // Act
    tracker.record(0, 0, hash_a, 1);
    tracker.record(0, 0, hash_a, 2);

    // Assert
    assert!(!tracker.has_quorum(0, 0, hash_b));
}

#[test]
fn snapshot_roundtrip_preserves_quorum() {
    // Arrange
    let mut tracker = VoteTracker::new(3);
    let hash = [7u8; 32];
    tracker.record(2, 1, hash, 10);
    tracker.record(2, 1, hash, 11);
    tracker.record(2, 1, hash, 12);
    let snapshot = tracker.snapshot();

    // Act
    let restored = VoteTracker::from_snapshot(3, snapshot);

    // Assert
    assert!(restored.has_quorum(2, 1, hash));
}

#[test]
fn from_snapshot_duplicate_voters_do_not_double_count() {
    // Arrange
    let hash = [8u8; 32];
    let mut snapshot = std::collections::BTreeMap::new();
    snapshot.insert((3, 0, hash), vec![1, 1, 2]);

    // Act
    let restored = VoteTracker::from_snapshot(3, snapshot);

    // Assert
    assert!(!restored.has_quorum(3, 0, hash));
}

#[test]
fn clear_then_snapshot_returns_empty() {
    // Arrange
    let mut tracker = VoteTracker::new(2);
    let hash = [9u8; 32];
    tracker.record(0, 0, hash, 1);
    tracker.clear();

    // Act
    let snapshot = tracker.snapshot();

    // Assert
    assert!(snapshot.is_empty());
}
