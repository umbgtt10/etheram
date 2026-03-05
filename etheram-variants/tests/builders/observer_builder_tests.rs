// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::observer::EventLevel;
use etheram_variants::builders::observer_builder::ObserverBuilder;
use etheram_variants::variants::ObserverVariant;

#[test]
fn build_missing_variant_returns_error() {
    // Arrange
    let builder = ObserverBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn build_with_no_op_variant_builds_successfully() {
    // Arrange
    let builder = ObserverBuilder::new().with_variant(ObserverVariant::NoOp);

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_no_op_observer() {
    // Arrange & Act
    let result = ObserverBuilder::default().build();

    // Assert
    let observer = result.unwrap();
    assert_eq!(observer.min_level(), EventLevel::None);
}
