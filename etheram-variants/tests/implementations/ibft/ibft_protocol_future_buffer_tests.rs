// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use barechain_core::collection::Collection;
use barechain_core::consensus_protocol::ConsensusProtocol;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::block::Block;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn handle_message_prepare_future_round_is_buffered_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let future_prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 5,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &future_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_future_round_is_buffered_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let future_commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 3,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &future_commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_current_round_is_not_buffered() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let current_prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &current_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_timeout_round_replays_buffered_pre_prepare() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let round_1_block = Block::empty(0, 1, ctx.state_root);
    let future_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block: round_1_block,
    });
    protocol.handle_message(&MessageSource::Peer(1), &future_pre_prepare, &ctx);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Assert
    let all_actions = actions.into_inner();
    let has_view_change = all_actions.iter().any(|a| {
        matches!(
            a,
            Action::BroadcastMessage {
                message: IbftMessage::ViewChange { round: 1, .. }
            }
        )
    });
    assert!(has_view_change);
    let has_prepare_from_replay = all_actions.iter().any(|a| {
        matches!(
            a,
            Action::BroadcastMessage {
                message: IbftMessage::Prepare { round: 1, .. }
            }
        )
    });
    assert!(has_prepare_from_replay);
}

#[test]
fn handle_message_pre_prepare_future_round_is_buffered_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let round_2_block = Block::empty(0, 2, ctx.state_root);
    let future_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 2,
        block: round_2_block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &future_pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
