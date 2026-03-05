// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::validator_set::ValidatorSet;

#[test]
fn quorum_size_four_validators_returns_three() {
    // Arrange
    let vs = ValidatorSet::new(vec![0, 1, 2, 3]);

    // Act & Assert
    assert_eq!(vs.quorum_size(), 3);
}

#[test]
fn contains_known_peer_returns_true() {
    // Arrange
    let vs = ValidatorSet::new(vec![10, 20, 30]);

    // Act & Assert
    assert!(vs.contains(20));
}

#[test]
fn contains_unknown_peer_returns_false() {
    // Arrange
    let vs = ValidatorSet::new(vec![10, 20, 30]);

    // Act & Assert
    assert!(!vs.contains(99));
}

#[test]
fn get_proposer_rotates_by_height() {
    // Arrange
    let vs = ValidatorSet::new(vec![10, 20, 30, 40]);

    // Act & Assert
    assert_eq!(vs.get_proposer(0), 10);
    assert_eq!(vs.get_proposer(1), 20);
    assert_eq!(vs.get_proposer(4), 10);
}

#[test]
fn get_proposer_for_round_rotates_by_height_and_round() {
    // Arrange
    let vs = ValidatorSet::new(vec![10, 20, 30, 40]);

    // Act & Assert
    assert_eq!(vs.get_proposer_for_round(0, 0), 10);
    assert_eq!(vs.get_proposer_for_round(0, 1), 20);
    assert_eq!(vs.get_proposer_for_round(1, 1), 30);
    assert_eq!(vs.get_proposer_for_round(3, 2), 20);
}

#[test]
fn new_empty_validator_set_quorum_and_count_match_current_semantics() {
    // Arrange & Act
    let vs = ValidatorSet::new(vec![]);

    // Assert
    assert_eq!(vs.quorum_size(), 1);
    assert_eq!(vs.count(), 0);
    assert_eq!(vs.validators(), Vec::<u64>::new());
}

#[test]
#[should_panic]
fn get_proposer_for_round_empty_validator_set_panics() {
    // Arrange
    let vs = ValidatorSet::new(vec![]);

    // Act
    let _ = vs.get_proposer_for_round(0, 0);

    // Assert
}

#[test]
fn quorum_size_five_validators_returns_four() {
    // Arrange
    let vs = ValidatorSet::new(vec![0, 1, 2, 3, 4]);

    // Act & Assert
    assert_eq!(vs.quorum_size(), 4);
}

#[test]
#[should_panic]
fn get_proposer_empty_validator_set_panics() {
    // Arrange
    let vs = ValidatorSet::new(vec![]);

    // Act & Assert
    vs.get_proposer(0);
}
