// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::builders::storage_builder::StorageBuilder;
use barechain_etheram_variants::variants::StorageVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = StorageBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_in_memory_builds_successfully() {
    // Arrange
    let builder = StorageBuilder::new();

    // Act
    let result = builder.with_variant(StorageVariant::InMemory).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = StorageBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
