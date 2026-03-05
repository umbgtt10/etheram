// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_variants::builders::partitioner_builder::PartitionerBuilder;
use etheram_variants::variants::PartitionerVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = PartitionerBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_type_based_builds_successfully() {
    // Arrange
    let builder = PartitionerBuilder::new();

    // Act
    let result = builder.with_variant(PartitionerVariant::TypeBased).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = PartitionerBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
