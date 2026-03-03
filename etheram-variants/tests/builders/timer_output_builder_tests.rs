// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::builders::timer_output_builder::TimerOutputBuilder;
use barechain_etheram_variants::variants::TimerOutputVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = TimerOutputBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_noop_builds_successfully() {
    // Arrange
    let builder = TimerOutputBuilder::new();

    // Act
    let result = builder.with_variant(TimerOutputVariant::NoOp).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = TimerOutputBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
