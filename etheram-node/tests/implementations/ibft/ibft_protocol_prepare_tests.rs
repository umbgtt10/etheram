// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::block::Block;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn handle_message_prepare_quorum_reached_broadcasts_commit() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Commit { .. }
        })
    ));
}

#[test]
fn handle_message_prepare_from_unknown_sender_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(99), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_extra_prepare_after_commit_sent_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_old_height_prepare_after_commit_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_wrong_block_hash_does_not_trigger_commit() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });
    protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);
    let wrong_hash = [0xffu8; 32];
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: wrong_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_without_pending_block_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block_hash = [0xaau8; 32];
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_wrong_height_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 1,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_wrong_round_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 1,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
