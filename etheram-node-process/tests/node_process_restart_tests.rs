// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node_process::etheram_node::NodeRuntime;
use etheram_node_process::infra::storage::sled_storage::SledStorage;
use etheram_node_process::infra::wal::file_wal::FileWal;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const RESTART_TIMEOUT_MS: u64 = 2_000;

fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to allocate local port");
    listener
        .local_addr()
        .expect("failed to get local socket address")
        .port()
}

fn wait_for_db_unlock(db_path: &str) {
    let started = Instant::now();
    loop {
        match SledStorage::new(db_path) {
            Ok(storage) => {
                drop(storage);
                return;
            }
            Err(_) if started.elapsed() <= Duration::from_millis(RESTART_TIMEOUT_MS) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("timed out waiting for db unlock: {}", error),
        }
    }
}

fn sample_locked_wal() -> ConsensusWal {
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
fn new_existing_db_restores_height_and_last_hash_across_restart() {
    // Arrange
    let db_path = create_test_db_path("node_runtime_restart");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let block = Block::empty(0, 1, [11u8; 32]);
    let expected_last_hash = block.compute_hash();
    storage.mutate(StorageMutation::StoreBlock(block));
    storage.mutate(StorageMutation::IncrementHeight);
    drop(storage);
    let transport_addr_1 = format!("127.0.0.1:{}", next_port());
    let client_addr_1 = format!("127.0.0.1:{}", next_port());
    let transport_addr_2 = format!("127.0.0.1:{}", next_port());
    let client_addr_2 = format!("127.0.0.1:{}", next_port());
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(1, transport_addr_1.clone());

    // Act
    let runtime_1 = NodeRuntime::new(
        1,
        &transport_addr_1,
        &client_addr_1,
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build first runtime");
    let height_1 = runtime_1.current_height();
    let hash_1 = runtime_1.last_block_hash();
    drop(runtime_1);
    wait_for_db_unlock(db_path.to_string_lossy().as_ref());
    peer_addresses.insert(1, transport_addr_2.clone());
    let runtime_2 = NodeRuntime::new(
        1,
        &transport_addr_2,
        &client_addr_2,
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build second runtime");

    // Assert
    assert_eq!(height_1, 1);
    assert_eq!(hash_1, expected_last_hash);
    assert_eq!(runtime_2.current_height(), 1);
    assert_eq!(runtime_2.last_block_hash(), expected_last_hash);

    cleanup_test_db_path(&db_path);
}

#[test]
fn new_seeded_wal_restores_locked_block_round_and_persists_across_restart() {
    // Arrange
    let db_path = create_test_db_path("node_runtime_wal_restart");
    let wal = FileWal::new(db_path.to_string_lossy().as_ref()).expect("failed to build file wal");
    let expected = sample_locked_wal();
    wal.store(&expected).expect("failed to seed wal");
    let transport_addr_1 = format!("127.0.0.1:{}", next_port());
    let client_addr_1 = format!("127.0.0.1:{}", next_port());
    let transport_addr_2 = format!("127.0.0.1:{}", next_port());
    let client_addr_2 = format!("127.0.0.1:{}", next_port());
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(1, transport_addr_1.clone());

    // Act
    let mut runtime_1 = NodeRuntime::new(
        1,
        &transport_addr_1,
        &client_addr_1,
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build first runtime");
    assert_eq!(runtime_1.run_steps(1), 1);
    drop(runtime_1);
    wait_for_db_unlock(db_path.to_string_lossy().as_ref());
    let persisted_after_first = wal
        .load()
        .expect("failed to reload wal after first runtime")
        .expect("missing wal after first runtime");
    peer_addresses.insert(1, transport_addr_2.clone());
    let mut runtime_2 = NodeRuntime::new(
        1,
        &transport_addr_2,
        &client_addr_2,
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build second runtime");
    assert_eq!(runtime_2.run_steps(1), 1);
    drop(runtime_2);
    wait_for_db_unlock(db_path.to_string_lossy().as_ref());
    let persisted_after_second = wal
        .load()
        .expect("failed to reload wal after second runtime")
        .expect("missing wal after second runtime");

    // Assert
    assert_eq!(persisted_after_first.round, expected.round);
    assert!(persisted_after_first.prepare_sent);
    assert_eq!(
        persisted_after_first
            .pending_block
            .expect("missing pending block after first runtime")
            .compute_hash(),
        expected
            .pending_block
            .clone()
            .expect("missing expected pending block")
            .compute_hash()
    );
    assert_eq!(persisted_after_second.round, expected.round);
    assert!(persisted_after_second.prepare_sent);
    assert_eq!(
        persisted_after_second
            .pending_block
            .expect("missing pending block after second runtime")
            .compute_hash(),
        expected
            .pending_block
            .expect("missing expected pending block")
            .compute_hash()
    );

    cleanup_test_db_path(&db_path);
}
