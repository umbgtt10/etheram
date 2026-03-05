// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_restored_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_with;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::block::Block;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn handle_message_pre_prepare_duplicate_same_sender_same_sequence_processed_once() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 7,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });

    // Act
    let first = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);
    let second = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(first.len(), 1);
    assert!(matches!(
        first.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
    assert_eq!(second.len(), 0);
}

#[test]
fn handle_message_same_sequence_different_message_kind_remains_independent() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 8,
        height: 0,
        round: 0,
        block,
    });
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 8,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let first = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);
    let second = protocol.handle_message(&MessageSource::Peer(0), &prepare, &ctx);

    // Assert
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_seen_message_duplicate_is_ignored() {
    // Arrange
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 9,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    let wal = setup_wal_with(|wal| {
        wal.seen_messages = vec![(0, 0, 0, 9)];
    });
    let mut protocol = setup_restored_protocol(wal);
    let ctx = setup_context(1, 0);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_duplicate_same_sender_same_sequence_does_not_double_count() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 10,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);

    // Act
    let duplicate = protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    let quorum = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 11,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(duplicate.len(), 0);
    assert_eq!(quorum.len(), 1);
    assert!(matches!(
        quorum.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Commit { .. }
        })
    ));
}

#[test]
fn handle_message_commit_duplicate_same_sender_same_sequence_does_not_finalize_early() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 12,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 13,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 14,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Act
    let duplicate = protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);
    let quorum = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Commit {
            sequence: 15,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(duplicate.len(), 0);
    assert_eq!(quorum.len(), 3);
    assert!(matches!(quorum.get(0), Some(Action::ExecuteBlock { .. })));
    assert!(matches!(quorum.get(1), Some(Action::StoreBlock { .. })));
    assert!(matches!(quorum.get(2), Some(Action::IncrementHeight)));
}

#[test]
fn handle_message_same_sequence_same_kind_different_sender_is_not_deduplicated() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 21,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_from_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 21,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let first = protocol.handle_message(&MessageSource::Peer(1), &prepare_from_1, &ctx);
    let second = protocol.handle_message(&MessageSource::Peer(2), &prepare_from_2, &ctx);

    // Assert
    assert_eq!(first.len(), 0);
    assert_eq!(second.len(), 1);
    assert!(matches!(
        second.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Commit { .. }
        })
    ));
}

#[test]
fn handle_message_invalid_then_duplicate_invalid_pre_prepare_is_idempotent_noop() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let invalid = Message::Peer(IbftMessage::PrePrepare {
        sequence: 30,
        height: 1,
        round: 0,
        block: Block::new(1, 0, vec![], [0u8; 32]),
    });

    // Act
    let first = protocol.handle_message(&MessageSource::Peer(0), &invalid, &ctx);
    let second = protocol.handle_message(&MessageSource::Peer(0), &invalid, &ctx);

    // Assert
    assert_eq!(first.len(), 0);
    assert_eq!(second.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_seen_message_non_duplicate_is_accepted() {
    // Arrange
    let wal = setup_wal_with(|wal| {
        wal.seen_messages = vec![(0, 0, 0, 8)];
    });
    let mut protocol = setup_restored_protocol(wal);
    let ctx = setup_context(1, 0);
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 9,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}
