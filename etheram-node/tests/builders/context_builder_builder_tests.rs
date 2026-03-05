// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::{
    builders::context_builder_builder::ContextBuilderBuilder, variants::ContextBuilderVariant,
};

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = ContextBuilderBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_eager_builds_successfully() {
    // Arrange
    let builder = ContextBuilderBuilder::new();

    // Act
    let result = builder.with_variant(ContextBuilderVariant::Eager).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = ContextBuilderBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
