// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::build_block_with_commitments;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_restored_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_with;
use alloc::collections::BTreeMap;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_node::incoming::timer::timer_event::TimerEvent;

#[test]
fn handle_message_pre_prepare_conflicting_block_same_height_round_same_sender_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let accepted = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    let conflicting = Message::Peer(IbftMessage::PrePrepare {
        sequence: 2,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [1u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &accepted, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &conflicting, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_conflicting_block_new_sequence_same_sender_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let first = Message::Peer(IbftMessage::PrePrepare {
        sequence: 10,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    let second = Message::Peer(IbftMessage::PrePrepare {
        sequence: 11,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [2u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &first, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &second, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_prepare_for_rejected_conflict_hash_does_not_emit_commit() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert([1u8; 20], Account::new(100));
    let accepted_block = Block::new(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 0)],
        [0u8; 32],
    );
    let accepted_hash = accepted_block.compute_hash();
    let conflicting_block = Block::new(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [3u8; 20], 2, 21_000, 0)],
        [0u8; 32],
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 20,
            height: 0,
            round: 0,
            block: accepted_block,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 21,
            height: 0,
            round: 0,
            block: conflicting_block,
        }),
        &ctx,
    );

    // Act
    let prepare_from_0 = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 30,
            height: 0,
            round: 0,
            block_hash: accepted_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let prepare_from_2 = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 31,
            height: 0,
            round: 0,
            block_hash: accepted_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let prepare_from_3 = protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 32,
            height: 0,
            round: 0,
            block_hash: accepted_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(prepare_from_0.len(), 0);
    assert_eq!(prepare_from_2.len(), 0);
    assert_eq!(prepare_from_3.len(), 0);
}

#[test]
fn handle_message_commit_for_rejected_conflict_hash_does_not_store_block() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let mut commit_votes = BTreeMap::new();
    commit_votes.insert((0, 0, block_hash), vec![0, 2]);
    let wal = setup_wal_with(|wal| {
        wal.pending_block = Some(block);
        wal.commit_votes = commit_votes;
        wal.rejected_block_hashes = vec![(0, 0, block_hash)];
        wal.prepare_sent = true;
        wal.commit_sent = true;
    });
    let mut protocol = setup_restored_protocol(wal);
    let ctx = setup_context(1, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Commit {
            sequence: 40,
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
fn consensus_wal_malicious_tracking_roundtrip_restores_rejection_behavior() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let accepted_block = Block::new(0, 0, vec![], [0u8; 32]);
    let accepted_hash = accepted_block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 50,
            height: 0,
            round: 0,
            block: accepted_block,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 51,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [4u8; 32]),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored = setup_restored_protocol(wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 60,
            height: 0,
            round: 0,
            block_hash: accepted_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_invalid_state_root_conflict_from_proposer_then_valid_pre_prepare_is_accepted() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let invalid = Message::Peer(IbftMessage::PrePrepare {
        sequence: 70,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [9u8; 32]),
    });
    let valid = Message::Peer(IbftMessage::PrePrepare {
        sequence: 71,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &valid, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn handle_message_wrong_height_conflicts_do_not_mark_rejected_hashes() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let stale_a = Message::Peer(IbftMessage::PrePrepare {
        sequence: 72,
        height: 1,
        round: 0,
        block: Block::new(1, 0, vec![], [0u8; 32]),
    });
    let stale_b = Message::Peer(IbftMessage::PrePrepare {
        sequence: 73,
        height: 1,
        round: 0,
        block: Block::new(1, 0, vec![], [7u8; 32]),
    });
    let valid = Message::Peer(IbftMessage::PrePrepare {
        sequence: 74,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [0u8; 32]),
    });
    protocol.handle_message(&MessageSource::Peer(0), &stale_a, &ctx);
    protocol.handle_message(&MessageSource::Peer(0), &stale_b, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &valid, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn handle_message_restart_after_invalid_conflict_noise_then_valid_path_progresses() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 75,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [8u8; 32]),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 76,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [9u8; 32]),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored = setup_restored_protocol(wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 77,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [0u8; 32]),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn handle_message_malicious_sender_prepare_does_not_help_reach_quorum() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert([1u8; 20], Account::new(100));
    let contract_storage = BTreeMap::new();
    let first = build_block_with_commitments(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 0)],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let second = build_block_with_commitments(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [3u8; 20], 2, 21_000, 0)],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 78,
            height: 0,
            round: 0,
            block: first,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 79,
            height: 0,
            round: 0,
            block: second,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let round_one_hash = Block::new(0, 1, vec![], [0u8; 32]).compute_hash();

    // Act
    let from_malicious = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 80,
            height: 0,
            round: 1,
            block_hash: round_one_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    let from_honest = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 81,
            height: 0,
            round: 1,
            block_hash: round_one_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(from_malicious.len(), 0);
    assert_eq!(from_honest.len(), 0);
}

#[test]
fn handle_message_restore_from_wal_malicious_sender_prepare_stays_ignored() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let mut prepare_votes = BTreeMap::new();
    prepare_votes.insert((0, 0, block_hash), vec![1, 2]);
    let wal = setup_wal_with(|wal| {
        wal.pending_block = Some(block);
        wal.prepare_votes = prepare_votes;
        wal.malicious_senders = vec![(0, 0)];
        wal.prepare_sent = true;
    });
    let mut protocol = setup_restored_protocol(wal);
    let ctx = setup_context(1, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 82,
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
fn handle_message_malicious_sender_view_change_does_not_help_reach_new_view_quorum() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert([1u8; 20], Account::new(100));
    let contract_storage = BTreeMap::new();
    let first = build_block_with_commitments(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 0)],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let second = build_block_with_commitments(
        0,
        0,
        vec![Transaction::transfer([1u8; 20], [3u8; 20], 2, 21_000, 0)],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 83,
            height: 0,
            round: 0,
            block: first,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 84,
            height: 0,
            round: 0,
            block: second,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Act
    let from_malicious = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 85,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );
    let from_honest = protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 86,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &ctx,
    );

    // Assert
    assert_eq!(from_malicious.len(), 0);
    assert_eq!(from_honest.len(), 0);
}

#[test]
fn handle_message_malicious_sender_new_view_is_ignored() {
    // Arrange
    let wal = setup_wal_with(|wal| {
        wal.round = 1;
        wal.prepare_sent = true;
        wal.pending_block = Some(Block::new(0, 1, vec![], [0u8; 32]));
        wal.malicious_senders = vec![(0, 0)];
        wal.view_change_votes.insert((0, 1), vec![0, 1, 2]);
    });
    let mut protocol = setup_restored_protocol(wal);
    let ctx = setup_context(2, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 90,
        height: 0,
        round: 1,
        block: Block::new(0, 1, vec![], [0u8; 32]),
    });

    // Act
    let _ = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::NewView {
            sequence: 89,
            height: 0,
            round: 1,
            prepared_certificate: None,
            view_change_senders: vec![0, 1, 2],
        }),
        &ctx,
    );
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_pre_prepare_with_oversized_gas_limit_is_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(
        from,
        Account {
            balance: 1_000,
            nonce: 0,
        },
    );
    let oversized_tx = Transaction::transfer(from, [2u8; 20], 100, 2_000_000, 0);
    let block = Block::new(0, 0, vec![oversized_tx], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
