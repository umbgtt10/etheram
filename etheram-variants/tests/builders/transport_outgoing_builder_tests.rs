// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_variants::builders::transport_outgoing_builder::TransportOutgoingBuilder;
use etheram_variants::variants::OutgoingTransportVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = TransportOutgoingBuilder::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_noop_builds_successfully() {
    // Arrange
    let builder = TransportOutgoingBuilder::new();

    // Act
    let result = builder.with_variant(OutgoingTransportVariant::NoOp).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = TransportOutgoingBuilder::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
