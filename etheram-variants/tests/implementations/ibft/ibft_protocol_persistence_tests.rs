// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_with;
use barechain_core::collection::Collection;
use barechain_core::consensus_protocol::ConsensusProtocol;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use barechain_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use barechain_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::block::Block;
use etheram::context::context_dto::Context;
use etheram::incoming::timer::timer_event::TimerEvent;
use std::collections::BTreeMap;

fn valid_pre_prepare_message(sequence: u64, block: Block) -> Message<IbftMessage> {
    Message::Peer(IbftMessage::PrePrepare {
        sequence,
        height: block.height,
        round: 0,
        block,
    })
}

#[test]
fn handle_message_restore_from_wal_after_timeout_round_preserves_round_proposer_logic() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let proposer_ctx = Context::new(0, 0, [0u8; 32]);
    let restored_proposer_ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &proposer_ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &restored_proposer_ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { round: 1, .. }
        })
    ));
}

#[test]
fn handle_message_restore_from_wal_after_prepare_sent_rejects_second_pre_prepare_vote() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(1, block.clone()),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(2, block),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_after_partial_prepare_votes_reaches_commit_quorum() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(1, block),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 1,
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
            sequence: 1,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(3),
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
            message: IbftMessage::Commit { .. }
        })
    ));
}

#[test]
fn handle_message_restore_from_wal_after_partial_commit_votes_stores_block_on_quorum() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(1, block),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 2,
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
            sequence: 2,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Commit {
            sequence: 3,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let first_actions = restored.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Commit {
            sequence: 3,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let second_actions = restored.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Commit {
            sequence: 3,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(first_actions.len(), 0);
    assert_eq!(second_actions.len(), 3);
    assert!(matches!(
        second_actions.get(0),
        Some(Action::ExecuteBlock { .. })
    ));
    assert!(matches!(
        second_actions.get(1),
        Some(Action::StoreBlock { .. })
    ));
    assert!(matches!(
        second_actions.get(2),
        Some(Action::IncrementHeight)
    ));
}

#[test]
fn handle_message_restore_from_wal_with_replay_state_rejects_lower_or_equal_sequences() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(5, block),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 4,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let prepare_actions = restored.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 4,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let pre_prepare_actions = restored.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(3, Block::new(0, 0, vec![], [0u8; 32])),
        &ctx,
    );

    // Assert
    assert_eq!(prepare_actions.len(), 0);
    assert_eq!(pre_prepare_actions.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_after_timeout_keeps_outgoing_sequence_increasing() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { sequence: 1, .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { sequence: 2, .. }
        })
    ));
}

#[test]
fn handle_message_restore_from_wal_with_round_one_rejects_round_zero_pre_prepare() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 4,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [0u8; 32]),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_with_partial_view_change_quorum_broadcasts_new_view() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 7,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 7,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView { round: 1, .. }
        })
    ));
}

#[test]
fn handle_message_restore_from_wal_after_new_view_sent_does_not_broadcast_duplicate_new_view() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 8,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 8,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 8,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_with_prepared_certificate_rejects_mismatched_view_change() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &valid_pre_prepare_message(1, block),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 3,
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
            sequence: 3,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);
    let mismatched = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [9u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 10,
            height: 0,
            round: 1,
            prepared_certificate: Some(mismatched),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn consensus_wal_roundtrip_multiple_times_preserves_defaults_and_behavior() {
    // Arrange
    let protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let wal = protocol.consensus_wal();
    let restored_once =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);
    let wal_again = restored_once.consensus_wal();
    let mut restored_twice = IbftProtocol::from_wal(
        vec![0, 1, 2, 3],
        Box::new(MockSignatureScheme::new(0)),
        wal_again,
    );
    let ctx = Context::new(0, 0, [0u8; 32]);

    // Act
    let actions = restored_twice.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { sequence: 0, .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { sequence: 1, .. }
        })
    ));
}

#[test]
fn consensus_wal_new_protocol_defaults_are_empty_and_round_zero() {
    // Arrange
    let protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));

    // Act
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.height, 0);
    assert_eq!(wal.round, 0);
    assert!(wal.pending_block.is_none());
    assert!(wal.prepared_certificate.is_none());
    assert!(wal.prepare_votes.is_empty());
    assert!(wal.commit_votes.is_empty());
    assert!(wal.view_change_votes.is_empty());
    assert!(wal.seen_messages.is_empty());
    assert!(wal.highest_seen_sequence.is_empty());
    assert!(!wal.prepare_sent);
    assert!(!wal.commit_sent);
    assert!(wal.new_view_sent_round.is_none());
    assert_eq!(wal.next_outgoing_sequence, 0);
}

#[test]
fn handle_message_restore_from_wal_with_max_seen_sequence_rejects_equal_prepare_sequence() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let mut highest_seen_sequence = BTreeMap::new();
    highest_seen_sequence.insert((2, 1), u64::MAX - 1);
    let mut prepare_votes = BTreeMap::new();
    prepare_votes.insert((0, 0, block_hash), vec![1]);
    let wal = setup_wal_with(|wal| {
        wal.pending_block = Some(block);
        wal.prepare_votes = prepare_votes;
        wal.highest_seen_sequence = highest_seen_sequence;
        wal.prepare_sent = true;
    });
    let mut protocol =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);
    let ctx = Context::new(1, 0, [0u8; 32]);

    // Act
    let rejected = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: u64::MAX - 1,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let accepted = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: u64::MAX,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(rejected.len(), 0);
    assert_eq!(accepted.len(), 0);
}

#[test]
fn handle_message_restore_from_inconsistent_wal_commit_sent_true_with_no_votes_is_safe() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let wal = setup_wal_with(|wal| {
        wal.pending_block = Some(block);
        wal.prepare_sent = true;
        wal.commit_sent = true;
    });
    let mut protocol =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);
    let ctx = Context::new(1, 0, [0u8; 32]);

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
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_height_two_rejects_stale_height_and_accepts_current_height() {
    // Arrange
    let stale_block = Block::new(1, 1, vec![], [0u8; 32]);
    let current_block = Block::new(2, 2, vec![], [0u8; 32]);
    let wal = setup_wal_with(|wal| {
        wal.height = 2;
    });
    let mut protocol =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);
    let ctx = Context::new(1, 2, [0u8; 32]);

    // Act
    let stale_actions = protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 10,
            height: 1,
            round: 0,
            block: stale_block,
        }),
        &ctx,
    );
    let current_actions = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 11,
            height: 2,
            round: 0,
            block: current_block,
        }),
        &ctx,
    );

    // Assert
    assert_eq!(stale_actions.len(), 0);
    assert_eq!(current_actions.len(), 1);
    assert!(matches!(
        current_actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 2,
                round: 0,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_restore_from_wal_preserves_prepare_signatures() {
    // Arrange
    let sig = SignatureBytes::from_slice(&[0xeeu8; 96]);
    let wal = setup_wal_with(|w| {
        w.prepare_signatures = vec![(0, 0, 3, sig)];
    });
    let protocol =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let restored_wal = protocol.consensus_wal();

    // Assert
    assert_eq!(restored_wal.prepare_signatures.len(), 1);
    let (h, r, p, s) = restored_wal.prepare_signatures[0];
    assert_eq!(h, 0);
    assert_eq!(r, 0);
    assert_eq!(p, 3);
    assert_eq!(s, sig);
}

#[test]
fn consensus_wal_after_propose_block_contains_self_prepare_signature() {
    // Arrange & Act
    let (protocol, _, _) = setup_after_propose();
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.prepare_signatures.len(), 1);
    let (h, r, p, _) = wal.prepare_signatures[0];
    assert_eq!(h, 0);
    assert_eq!(r, 0);
    assert_eq!(p, 0);
}

#[test]
fn consensus_wal_after_timeout_round_preserves_prepare_signatures() {
    // Arrange
    let (mut protocol, ctx, _) = setup_after_propose();

    // Act
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.prepare_signatures.len(), 1);
}

#[test]
fn consensus_wal_after_commit_clears_prepare_signatures() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 10,
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
            sequence: 11,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Commit {
            sequence: 12,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Commit {
            sequence: 13,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Act
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Commit {
            sequence: 14,
            height: 0,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();

    // Assert
    assert!(wal.prepare_signatures.is_empty());
}
