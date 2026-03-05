// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_base;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_with;
use alloc::collections::BTreeMap;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;

fn round_trip(wal: ConsensusWal) -> ConsensusWal {
    let bytes = wal.to_bytes();
    ConsensusWal::from_bytes(&bytes).expect("round-trip deserialization failed")
}

fn tx(value: u128) -> Transaction {
    Transaction::transfer([1u8; 20], [2u8; 20], value, 21_000, 0)
}

fn block_with_tx() -> Block {
    Block::new(1, 0, vec![tx(500)], [0xabu8; 32])
}

#[test]
fn to_bytes_from_bytes_empty_wal_preserves_all_fields() {
    // Arrange
    let wal = setup_wal_base();

    // Act
    let restored = round_trip(wal.clone());

    // Assert
    assert_eq!(restored.height, wal.height);
    assert_eq!(restored.round, wal.round);
    assert_eq!(restored.active_validators, wal.active_validators);
    assert!(restored.pending_block.is_none());
    assert!(restored.prepared_certificate.is_none());
    assert!(!restored.prepare_sent);
    assert!(!restored.commit_sent);
    assert_eq!(restored.new_view_sent_round, None);
    assert_eq!(restored.next_outgoing_sequence, 0);
}

#[test]
fn to_bytes_from_bytes_height_and_round_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.height = 42;
        w.round = 3;
        w.next_outgoing_sequence = 17;
        w.prepare_sent = true;
        w.commit_sent = true;
        w.new_view_sent_round = Some(2);
    });

    // Act
    let restored = round_trip(wal.clone());

    // Assert
    assert_eq!(restored.height, 42);
    assert_eq!(restored.round, 3);
    assert_eq!(restored.next_outgoing_sequence, 17);
    assert!(restored.prepare_sent);
    assert!(restored.commit_sent);
    assert_eq!(restored.new_view_sent_round, Some(2));
}

#[test]
fn to_bytes_from_bytes_pending_block_with_transactions_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.pending_block = Some(block_with_tx());
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    let block = restored.pending_block.expect("pending_block missing");
    assert_eq!(block.height, 1);
    assert_eq!(block.proposer, 0);
    assert_eq!(block.state_root, [0xabu8; 32]);
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(block.transactions[0].value, 500);
    assert_eq!(block.transactions[0].from, [1u8; 20]);
}

#[test]
fn to_bytes_from_bytes_prepared_certificate_preserved() {
    // Arrange
    let cert = PreparedCertificate {
        height: 5,
        round: 1,
        block_hash: [0xccu8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let wal = setup_wal_with(|w| {
        w.prepared_certificate = Some(cert.clone());
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    let rc = restored
        .prepared_certificate
        .expect("prepared_certificate missing");
    assert_eq!(rc.height, 5);
    assert_eq!(rc.round, 1);
    assert_eq!(rc.block_hash, [0xccu8; 32]);
    assert_eq!(
        rc.signed_prepares,
        vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ]
    );
}

#[test]
fn to_bytes_from_bytes_vote_maps_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.prepare_votes.insert((1, 0, [0xddu8; 32]), vec![0, 1, 2]);
        w.commit_votes
            .insert((1, 0, [0xeeu8; 32]), vec![0, 1, 2, 3]);
        w.view_change_votes.insert((1, 1), vec![0, 2]);
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    assert_eq!(
        restored.prepare_votes.get(&(1, 0, [0xddu8; 32])),
        Some(&vec![0u64, 1, 2])
    );
    assert_eq!(
        restored.commit_votes.get(&(1, 0, [0xeeu8; 32])),
        Some(&vec![0u64, 1, 2, 3])
    );
    assert_eq!(
        restored.view_change_votes.get(&(1, 1)),
        Some(&vec![0u64, 2])
    );
}

#[test]
fn to_bytes_from_bytes_rejected_and_malicious_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.rejected_block_hashes.push((3, 1, [0xffu8; 32]));
        w.malicious_senders.push((3, 2));
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    assert_eq!(restored.rejected_block_hashes, vec![(3, 1, [0xffu8; 32])]);
    assert_eq!(restored.malicious_senders, vec![(3, 2)]);
}

#[test]
fn to_bytes_from_bytes_seen_messages_and_sequence_tracking_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.seen_messages.push((1, 0, 2, 99));
        w.highest_seen_sequence.insert((0, 2), 99);
        w.observed_pre_prepares.insert((1, 0, 0), [0xbbu8; 32]);
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    assert_eq!(restored.seen_messages, vec![(1, 0, 2, 99)]);
    assert_eq!(restored.highest_seen_sequence.get(&(0, 2)), Some(&99));
    assert_eq!(
        restored.observed_pre_prepares.get(&(1, 0, 0)),
        Some(&[0xbbu8; 32])
    );
}

#[test]
fn to_bytes_from_bytes_scheduled_validator_updates_preserved() {
    // Arrange
    let wal = setup_wal_with(|w| {
        w.scheduled_validator_updates.insert(5, vec![0, 1, 2, 3, 4]);
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    let mut expected = BTreeMap::new();
    expected.insert(5u64, vec![0u64, 1, 2, 3, 4]);
    assert_eq!(restored.scheduled_validator_updates, expected);
}

#[test]
fn from_bytes_returns_none_for_truncated_input() {
    // Arrange
    let wal = setup_wal_base();
    let bytes = wal.to_bytes();
    let truncated = &bytes[..bytes.len() / 2];

    // Act & Assert
    assert!(ConsensusWal::from_bytes(truncated).is_none());
}

#[test]
fn from_bytes_returns_none_for_empty_input() {
    // Act & Assert
    assert!(ConsensusWal::from_bytes(&[]).is_none());
}

#[test]
fn to_bytes_from_bytes_prepare_signatures_preserved() {
    // Arrange
    let sig = SignatureBytes::from_slice(&[0xddu8; 96]);
    let wal = setup_wal_with(|w| {
        w.prepare_signatures = vec![(1, 0, 2, sig)];
    });

    // Act
    let restored = round_trip(wal);

    // Assert
    assert_eq!(restored.prepare_signatures.len(), 1);
    let (h, r, p, s) = restored.prepare_signatures[0];
    assert_eq!(h, 1);
    assert_eq!(r, 0);
    assert_eq!(p, 2);
    assert_eq!(s, sig);
}
