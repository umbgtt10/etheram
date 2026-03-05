// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_core::node_common::action_collection::ActionCollection;

#[test]
fn new_empty_collection_is_empty() {
    // Act & Assert
    assert!(ActionCollection::<u32>::new().is_empty());
}

#[test]
fn push_single_item_len_returns_one() {
    // Arrange
    let mut collection = ActionCollection::<u32>::new();

    // Act
    collection.push(42);

    // Assert
    assert_eq!(collection.len(), 1);
}

#[test]
fn iter_pushed_items_returns_all_items() {
    // Arrange
    let mut collection = ActionCollection::<u32>::new();
    collection.push(1);
    collection.push(2);
    collection.push(3);

    // Act
    let items: Vec<&u32> = collection.iter().collect();

    // Assert
    assert_eq!(items, vec![&1, &2, &3]);
}

#[test]
fn clear_populated_collection_becomes_empty() {
    // Arrange
    let mut collection = ActionCollection::<u32>::new();
    collection.push(1);
    collection.push(2);

    // Act
    collection.clear();

    // Assert
    assert!(collection.is_empty());
}

#[test]
fn from_vec_with_items_len_matches_source() {
    // Arrange
    let items = vec![10u32, 20, 30];

    // Act
    let collection = ActionCollection::from_vec(items);

    // Assert
    assert_eq!(collection.len(), 3);
}
