// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::block::Block;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn handle_message_prepare_lower_sequence_after_higher_sequence_blocks_later_quorum() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let wrong_hash = [0xabu8; 32];
    let valid_prepare_from_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let invalid_high_sequence_prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 5,
        height: 0,
        round: 0,
        block_hash: wrong_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_lower_sequence_prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 4,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_prepare_from_3 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(2), &valid_prepare_from_2, &ctx);
    protocol.handle_message(
        &MessageSource::Peer(1),
        &invalid_high_sequence_prepare_from_1,
        &ctx,
    );
    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(1),
        &valid_lower_sequence_prepare_from_1,
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);

    protocol.handle_message(&MessageSource::Peer(3), &valid_prepare_from_3, &ctx);
}

#[test]
fn handle_message_pre_prepare_higher_sequence_after_invalid_message_is_accepted() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let invalid_block = Block::new(1, 0, vec![], [0u8; 32]);
    let valid_block = Block::new(0, 0, vec![], [0u8; 32]);
    let invalid_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 4,
        height: 1,
        round: 0,
        block: invalid_block,
    });
    let valid_higher_sequence_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 5,
        height: 0,
        round: 0,
        block: valid_block,
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_pre_prepare, &ctx);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_higher_sequence_pre_prepare,
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                sequence: _,
                height: 0,
                round: 0,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_same_sequence_prepare_and_commit_from_same_sender_are_independent() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 7,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_from_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 7,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let commit_from_1 = Message::Peer(IbftMessage::Commit {
        sequence: 7,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let commit_from_2 = Message::Peer(IbftMessage::Commit {
        sequence: 7,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare_from_1, &ctx);

    // Act
    protocol.handle_message(&MessageSource::Peer(2), &prepare_from_2, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &commit_from_1, &ctx);
    let actions = protocol.handle_message(&MessageSource::Peer(2), &commit_from_2, &ctx);

    // Assert
    assert_eq!(actions.len(), 3);
    assert!(matches!(actions.get(0), Some(Action::ExecuteBlock { .. })));
    assert!(matches!(actions.get(1), Some(Action::StoreBlock { .. })));
    assert!(matches!(actions.get(2), Some(Action::IncrementHeight)));
}

#[test]
fn handle_message_commit_lower_sequence_after_higher_invalid_commit_blocks_later_quorum() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_from_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare_from_1, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare_from_2, &ctx);
    let wrong_hash = [0xceu8; 32];
    let invalid_high_sequence_commit_from_1 = Message::Peer(IbftMessage::Commit {
        sequence: 5,
        height: 0,
        round: 0,
        block_hash: wrong_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_lower_sequence_commit_from_1 = Message::Peer(IbftMessage::Commit {
        sequence: 4,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_commit_from_2 = Message::Peer(IbftMessage::Commit {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(
        &MessageSource::Peer(1),
        &invalid_high_sequence_commit_from_1,
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &valid_lower_sequence_commit_from_1,
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_commit_from_2, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_lower_sequence_after_higher_invalid_blocks_quorum_new_view() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [0x11u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (0, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let invalid_high_sequence_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 5,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
    });
    let valid_lower_sequence_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 4,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    let valid_view_change_from_2 = Message::Peer(IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(
        &MessageSource::Peer(0),
        &invalid_high_sequence_view_change,
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_lower_sequence_view_change,
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_view_change_from_2, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_lower_sequence_after_higher_invalid_does_not_advance_round() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let invalid_high_sequence_new_view = Message::Peer(IbftMessage::NewView {
        sequence: 5,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1],
    });
    let valid_lower_sequence_new_view = Message::Peer(IbftMessage::NewView {
        sequence: 4,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let round_one_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 1,
        block,
    });
    protocol.handle_message(
        &MessageSource::Peer(1),
        &invalid_high_sequence_new_view,
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &valid_lower_sequence_new_view,
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &round_one_pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_high_invalid_from_one_sender_does_not_block_other_sender_low_sequence() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let wrong_hash = [0x42u8; 32];
    let invalid_high_sequence_prepare_from_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 9,
        height: 0,
        round: 0,
        block_hash: wrong_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_low_sequence_prepare_from_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_low_sequence_prepare_from_3 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(
        &MessageSource::Peer(1),
        &invalid_high_sequence_prepare_from_1,
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &valid_low_sequence_prepare_from_2,
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(3),
        &valid_low_sequence_prepare_from_3,
        &ctx,
    );

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
fn handle_message_timer_propose_emits_strictly_increasing_outgoing_sequences() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    let first_sequence = match actions.get(0) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { sequence, .. },
        }) => *sequence,
        _ => panic!("expected first message to be PrePrepare"),
    };
    let second_sequence = match actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { sequence, .. },
        }) => *sequence,
        _ => panic!("expected second message to be Prepare"),
    };
    assert!(second_sequence > first_sequence);
}

#[test]
fn handle_message_timeout_then_propose_keeps_outgoing_sequences_increasing() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);

    // Act
    let timeout_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let propose_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(timeout_actions.len(), 1);
    assert_eq!(propose_actions.len(), 2);
    let timeout_sequence = match timeout_actions.get(0) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::ViewChange { sequence, .. },
        }) => *sequence,
        _ => panic!("expected timeout to emit ViewChange"),
    };
    let pre_prepare_sequence = match propose_actions.get(0) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { sequence, .. },
        }) => *sequence,
        _ => panic!("expected propose to emit PrePrepare"),
    };
    let prepare_sequence = match propose_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { sequence, .. },
        }) => *sequence,
        _ => panic!("expected propose to emit Prepare"),
    };
    assert!(pre_prepare_sequence > timeout_sequence);
    assert!(prepare_sequence > pre_prepare_sequence);
}

#[test]
fn handle_message_timer_propose_still_works_after_many_peer_sequence_updates() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    let invalid_prepare_a = Message::Peer(IbftMessage::Prepare {
        sequence: 10,
        height: 0,
        round: 0,
        block_hash: [1u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    });
    let invalid_prepare_b = Message::Peer(IbftMessage::Prepare {
        sequence: 11,
        height: 0,
        round: 0,
        block_hash: [2u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    });
    let invalid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 7,
        height: 1,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(1), &invalid_prepare_a, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &invalid_prepare_b, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &invalid_view_change, &ctx);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_pre_prepare_lower_sequence_after_higher_invalid_is_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let invalid_high = Message::Peer(IbftMessage::PrePrepare {
        sequence: 8,
        height: 1,
        round: 0,
        block: Block::new(1, 0, vec![], [0u8; 32]),
    });
    let valid_lower = Message::Peer(IbftMessage::PrePrepare {
        sequence: 7,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_high, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &valid_lower, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_equal_sequence_duplicate_is_rejected() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let invalid_high = Message::Peer(IbftMessage::Prepare {
        sequence: 3,
        height: 0,
        round: 0,
        block_hash: [0x22u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_equal = Message::Peer(IbftMessage::Prepare {
        sequence: 3,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &invalid_high, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &valid_equal, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_equal_sequence_duplicate_is_rejected() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare_1, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare_2, &ctx);
    let invalid_high = Message::Peer(IbftMessage::Commit {
        sequence: 4,
        height: 0,
        round: 0,
        block_hash: [0x33u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_equal = Message::Peer(IbftMessage::Commit {
        sequence: 4,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let commit_2 = Message::Peer(IbftMessage::Commit {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &invalid_high, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &valid_equal, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &commit_2, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_equal_sequence_duplicate_is_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let invalid_high = Message::Peer(IbftMessage::ViewChange {
        sequence: 2,
        height: 0,
        round: 1,
        prepared_certificate: Some(PreparedCertificate {
            height: 0,
            round: 1,
            block_hash: [0x44u8; 32],
            signed_prepares: vec![
                (0, SignatureBytes::zeroed()),
                (0, SignatureBytes::zeroed()),
                (2, SignatureBytes::zeroed()),
            ],
        }),
    });
    let valid_equal = Message::Peer(IbftMessage::ViewChange {
        sequence: 2,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    let from_2 = Message::Peer(IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_high, &ctx);
    protocol.handle_message(&MessageSource::Peer(0), &valid_equal, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &from_2, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_equal_sequence_duplicate_is_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let invalid_high = Message::Peer(IbftMessage::NewView {
        sequence: 4,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1],
    });
    let valid_equal = Message::Peer(IbftMessage::NewView {
        sequence: 4,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });
    let round_one_pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 1,
        block: Block::new(0, 1, vec![], [0u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(1), &invalid_high, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &valid_equal, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &round_one_pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_same_sequence_as_prepare_from_same_sender_is_independent() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 7,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);
    let block_hash = Block::new(0, 0, vec![], [0u8; 32]).compute_hash();
    let prepare_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 7,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_3 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &prepare_1, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare_2, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &prepare_3, &ctx);

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
fn handle_message_pre_prepare_same_sequence_as_view_change_from_same_sender_is_independent() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 3,
        height: 0,
        round: 0,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 3,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn handle_message_prepare_same_sequence_as_new_view_from_same_sender_is_independent() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 6,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });
    protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 1,
        block,
    });
    protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);
    let prepare_0 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 1,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let prepare_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 6,
        height: 0,
        round: 1,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &prepare_0, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &prepare_1, &ctx);
    let prepare_3 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 1,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &prepare_3, &ctx);

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
fn handle_message_sequence_state_persists_across_height_increment_rejects_lower_old_sender_sequence(
) {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx0 = setup_context(0, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx0,
    );
    let block0 = Block::new(0, 0, vec![], [0u8; 32]);
    let hash0 = block0.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 10,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Commit {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Commit {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    let ctx1 = setup_context(0, 1);
    let block1 = Block::new(1, 1, vec![], [0u8; 32]);
    let hash1 = block1.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 1,
            height: 1,
            round: 0,
            block: block1,
        }),
        &ctx1,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 9,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_height_higher_sequence_from_same_sender_is_accepted() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx0 = setup_context(0, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx0,
    );
    let block0 = Block::new(0, 0, vec![], [0u8; 32]);
    let hash0 = block0.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 10,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Commit {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Commit {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash: hash0,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx0,
    );
    let ctx1 = setup_context(0, 1);
    let block1 = Block::new(1, 1, vec![], [0u8; 32]);
    let hash1 = block1.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 1,
            height: 1,
            round: 0,
            block: block1,
        }),
        &ctx1,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 11,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 2,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 2,
            height: 1,
            round: 0,
            block_hash: hash1,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx1,
    );

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
fn handle_message_commit_broadcast_after_prepare_quorum_has_sequence_greater_than_prior_outgoing() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Commit { sequence, .. }
        }) if *sequence > 1
    ));
}

#[test]
fn handle_message_new_view_broadcast_has_sequence_greater_than_prior_outgoing() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let timeout_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let timeout_sequence = match timeout_actions.get(0) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::ViewChange { sequence, .. },
        }) => *sequence,
        _ => panic!("expected timeout to emit ViewChange"),
    };
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView { sequence, .. }
        }) if *sequence > timeout_sequence
    ));
}

#[test]
fn handle_message_peer_message_with_timer_source_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Timer, &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_unknown_sender_high_sequence_does_not_block_validator_with_lower_sequence() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let unknown = Message::Peer(IbftMessage::Prepare {
        sequence: 100,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_1 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    let valid_2 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(99), &unknown, &ctx);
    protocol.handle_message(&MessageSource::Peer(1), &valid_1, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_2, &ctx);

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
fn handle_message_timer_message_with_peer_source_processes_timer() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}
