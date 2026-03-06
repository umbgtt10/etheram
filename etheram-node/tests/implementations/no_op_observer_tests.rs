// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::implementations::no_op_observer::NoOpObserver;
use etheram_node::observer::{ActionKind, EventLevel, Observer};

#[test]
fn min_level_returns_none() {
    // Arrange
    let observer = NoOpObserver;

    // Act
    let level = observer.min_level();

    // Assert
    assert_eq!(level, EventLevel::None);
}

#[test]
fn node_started_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.node_started(0);
}

#[test]
fn message_received_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.message_received(0, &MessageSource::Peer(1));
    observer.message_received(0, &MessageSource::Client(2));
    observer.message_received(0, &MessageSource::Timer);
}

#[test]
fn context_built_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.context_built(0, 5, [0u8; 32], 3);
}

#[test]
fn action_emitted_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.action_emitted(0, &ActionKind::BroadcastMessage);
    observer.action_emitted(0, &ActionKind::SendMessage { to: 1 });
    observer.action_emitted(0, &ActionKind::IncrementHeight);
}

#[test]
fn mutation_applied_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.mutation_applied(0, &ActionKind::UpdateAccount { address: [1u8; 20] });
    observer.mutation_applied(0, &ActionKind::TransactionOutOfGas { address: [1u8; 20] });
    observer.mutation_applied(0, &ActionKind::TransactionReverted { address: [1u8; 20] });
    observer.mutation_applied(
        0,
        &ActionKind::TransactionInvalidOpcode { address: [1u8; 20] },
    );
    observer.mutation_applied(
        0,
        &ActionKind::StoreReceipts {
            height: 1,
            success_count: 1,
            out_of_gas_count: 1,
            reverted_count: 1,
            invalid_opcode_count: 1,
        },
    );
    observer.mutation_applied(0, &ActionKind::IncrementHeight);
}

#[test]
fn output_executed_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.output_executed(0, &ActionKind::SendClientResponse { client_id: 99 });
    observer.output_executed(0, &ActionKind::BroadcastMessage);
}

#[test]
fn step_completed_does_not_panic() {
    // Arrange
    let mut observer = NoOpObserver;

    // Act & Assert
    observer.step_completed(0, true);
    observer.step_completed(0, false);
}

#[test]
fn boxed_observer_works_as_trait_object() {
    // Arrange
    let mut observer: Box<dyn Observer> = Box::new(NoOpObserver);

    // Act & Assert
    observer.node_started(0);
    observer.message_received(0, &MessageSource::Timer);
    observer.context_built(0, 0, [0u8; 32], 0);
    observer.action_emitted(0, &ActionKind::BroadcastMessage);
    observer.mutation_applied(0, &ActionKind::IncrementHeight);
    observer.output_executed(0, &ActionKind::SendClientResponse { client_id: 1 });
    observer.step_completed(0, true);
}
