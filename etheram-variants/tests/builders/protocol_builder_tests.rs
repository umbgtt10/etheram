// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::builders::protocol_builder::ProtocolBuilder;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::no_op_execution_engine::NoOpExecutionEngine;
use barechain_etheram_variants::implementations::no_op_protocol::NoOpProtocol;
use barechain_etheram_variants::variants::ProtocolVariant;

#[test]
fn build_without_variant_returns_error() {
    // Arrange
    let builder = ProtocolBuilder::<()>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_noop_builds_successfully() {
    // Arrange
    let builder = ProtocolBuilder::<()>::new();

    // Act
    let result = builder.with_variant(ProtocolVariant::NoOp).build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn default_builds_successfully() {
    // Arrange
    let builder = ProtocolBuilder::<()>::default();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn with_variant_ibft_without_execution_engine_returns_error() {
    // Arrange
    let validators = vec![1u64, 2, 3, 4];
    let builder = ProtocolBuilder::<IbftMessage>::new();

    // Act
    let result = builder
        .with_variant(ProtocolVariant::Ibft { validators })
        .build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn with_variant_ibft_with_execution_engine_builds_successfully() {
    // Arrange
    let validators = vec![1u64, 2, 3, 4];
    let builder = ProtocolBuilder::<IbftMessage>::new();

    // Act
    let result = builder
        .with_execution_engine(Box::new(NoOpExecutionEngine))
        .with_variant(ProtocolVariant::Ibft { validators })
        .build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn with_protocol_builds_successfully() {
    // Arrange
    let protocol = Box::new(NoOpProtocol::<()>::new());
    let builder = ProtocolBuilder::<()>::new();

    // Act
    let result = builder.with_protocol(protocol).build();

    // Assert
    assert!(result.is_ok());
}
