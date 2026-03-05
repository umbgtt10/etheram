// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::node_common::message_source::MessageSource;
use etheram_node::brain::protocol::message::Message;
use etheram_node::context::context_dto::Context;
use etheram_node::implementations::no_op_protocol::NoOpProtocol;
use etheram_node::incoming::timer::timer_event::TimerEvent;

#[test]
fn handle_message_returns_empty_collection() {
    // Arrange
    let source = MessageSource::Timer;
    let message = Message::<()>::Timer(TimerEvent::ProposeBlock);
    let context = Context::new(1, 0, [0u8; 32]);
    let mut protocol = NoOpProtocol::<()>::new();

    // Act
    let actions = protocol.handle_message(&source, &message, &context);

    // Assert
    assert!(actions.into_inner().is_empty());
}

#[test]
fn new_creates_instance_with_empty_response() {
    // Arrange
    let source = MessageSource::Timer;
    let message = Message::<()>::Timer(TimerEvent::TimeoutRound);
    let context = Context::new(2, 1, [1u8; 32]);

    // Act
    let mut protocol = NoOpProtocol::<()>::new();
    let actions = protocol.handle_message(&source, &message, &context);

    // Assert
    assert!(actions.into_inner().is_empty());
}

#[test]
fn default_creates_instance_with_empty_response() {
    // Arrange
    let source = MessageSource::Peer(42);
    let message = Message::<()>::Timer(TimerEvent::ProposeBlock);
    let context = Context::new(1, 0, [0u8; 32]);

    // Act
    let mut protocol = NoOpProtocol::<()>::default();
    let actions = protocol.handle_message(&source, &message, &context);

    // Assert
    assert!(actions.into_inner().is_empty());
}
