// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::context::context_dto::Context;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::ibft::wal_writer::NoOpWalWriter;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node_process::infra::protocol::protocol_factory::build_protocol;

#[test]
fn build_protocol_timer_propose_block_emits_non_zero_signature() {
    // Arrange
    let mut protocol =
        build_protocol(7, &[7], None, Box::new(NoOpWalWriter)).expect("failed to build protocol");
    let ctx = Context::new(7, 0, [0u8; 32]);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    let prepare_signature = actions
        .iter()
        .find_map(|action| match action {
            Action::BroadcastMessage {
                message:
                    IbftMessage::Prepare {
                        sender_signature, ..
                    },
            } => Some(*sender_signature),
            _ => None,
        })
        .expect("expected prepare message");
    assert_ne!(prepare_signature, SignatureBytes::zeroed());
}
