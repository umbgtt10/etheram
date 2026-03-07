// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_node::common_types::block::Block;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node_process::infra::wal::file_wal::FileWal;
use std::collections::BTreeMap;
use std::fs;

fn sample_wal() -> ConsensusWal {
    let pending_block = Block::empty(0, 1, [7u8; 32]);
    ConsensusWal {
        height: 0,
        round: 5,
        active_validators: vec![1],
        scheduled_validator_updates: BTreeMap::new(),
        pending_block: Some(pending_block.clone()),
        observed_pre_prepares: BTreeMap::new(),
        prepared_certificate: Some(PreparedCertificate {
            height: 0,
            round: 5,
            block_hash: pending_block.compute_hash(),
            signed_prepares: vec![(1, SignatureBytes::zeroed())],
        }),
        prepare_votes: BTreeMap::new(),
        commit_votes: BTreeMap::new(),
        rejected_block_hashes: Vec::new(),
        malicious_senders: Vec::new(),
        view_change_votes: BTreeMap::new(),
        seen_messages: Vec::new(),
        highest_seen_sequence: BTreeMap::new(),
        prepare_sent: false,
        commit_sent: false,
        new_view_sent_round: None,
        next_outgoing_sequence: 0,
        prepare_signatures: Vec::new(),
    }
}

#[test]
fn load_missing_file_returns_none() {
    // Arrange
    let db_path = create_test_db_path("file_wal_missing");
    let wal = FileWal::new(db_path.to_string_lossy().as_ref()).expect("failed to build file wal");

    // Act
    let loaded = wal.load().expect("failed to load wal");

    // Assert
    assert!(loaded.is_none());
    cleanup_test_db_path(&db_path);
}

#[test]
fn store_then_load_round_trips_consensus_wal() {
    // Arrange
    let db_path = create_test_db_path("file_wal_round_trip");
    let wal = FileWal::new(db_path.to_string_lossy().as_ref()).expect("failed to build file wal");
    let expected = sample_wal();

    // Act
    wal.store(&expected).expect("failed to store wal");
    let loaded = wal
        .load()
        .expect("failed to load wal")
        .expect("missing wal");

    // Assert
    assert_eq!(loaded.height, expected.height);
    assert_eq!(loaded.round, expected.round);
    assert_eq!(loaded.active_validators, expected.active_validators);
    assert_eq!(
        loaded
            .pending_block
            .expect("missing pending block")
            .compute_hash(),
        expected
            .pending_block
            .expect("missing expected pending block")
            .compute_hash()
    );
    assert_eq!(
        loaded
            .prepared_certificate
            .expect("missing prepared certificate")
            .block_hash,
        expected
            .prepared_certificate
            .expect("missing expected prepared certificate")
            .block_hash
    );
    cleanup_test_db_path(&db_path);
}

#[test]
fn load_invalid_bytes_returns_error() {
    // Arrange
    let db_path = create_test_db_path("file_wal_invalid");
    let wal = FileWal::new(db_path.to_string_lossy().as_ref()).expect("failed to build file wal");
    fs::write(wal.path(), [1u8, 2, 3, 4]).expect("failed to write invalid wal bytes");

    // Act
    let result = wal.load();

    // Assert
    assert!(result.is_err());
    cleanup_test_db_path(&db_path);
}
