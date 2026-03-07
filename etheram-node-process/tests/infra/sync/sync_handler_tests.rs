// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_in_memory_storage::TestInMemoryStorage;
use etheram_node::builders::cache_builder::CacheBuilder;
use etheram_node::builders::storage_builder::StorageBuilder;
use etheram_node::common_types::block::Block;
use etheram_node::state::etheram_state::EtheramState;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node_process::infra::sync::sync_handler::SyncHandler;
use etheram_node_process::infra::sync::sync_message::SyncMessage;
use etheram_node_process::infra::sync::sync_sender::SyncSender;
use etheram_node_process::infra::transport::grpc_transport::sync_bus::SyncBus;
use std::sync::Arc;
use std::sync::Mutex;

type Broadcasts = Vec<(u64, [u8; 32])>;
type Sends = Vec<(u64, SyncMessage)>;

#[derive(Clone, Default)]
struct FakeSyncLog {
    sends: Arc<Mutex<Sends>>,
    broadcasts: Arc<Mutex<Broadcasts>>,
}

struct FakeSyncSender {
    log: FakeSyncLog,
}

impl FakeSyncSender {
    fn new(log: FakeSyncLog) -> Self {
        Self { log }
    }
}

impl SyncSender for FakeSyncSender {
    fn broadcast_status(&self, height: u64, last_hash: [u8; 32]) {
        self.log
            .broadcasts
            .lock()
            .expect("lock")
            .push((height, last_hash));
    }

    fn send_to_peer(&self, peer_id: u64, message: &SyncMessage) {
        self.log
            .sends
            .lock()
            .expect("lock")
            .push((peer_id, message.clone()));
    }
}

fn build_state() -> EtheramState {
    let storage = StorageBuilder::default()
        .build()
        .expect("failed to build storage");
    let cache = CacheBuilder::default()
        .build()
        .expect("failed to build cache");
    EtheramState::new(storage, cache)
}

fn build_state_with_blocks(blocks: &[Block]) -> EtheramState {
    let storage = StorageBuilder::default()
        .build()
        .expect("failed to build storage");
    let cache = CacheBuilder::default()
        .build()
        .expect("failed to build cache");
    let mut state = EtheramState::new(storage, cache);
    for block in blocks {
        state.apply_single_mutation(StorageMutation::StoreBlock(block.clone()));
        state.apply_single_mutation(StorageMutation::IncrementHeight);
    }
    state
}

#[test]
fn process_sync_messages_empty_queue_does_nothing() {
    // Arrange
    let peer_id = 1;
    let sync_bus = Arc::new(SyncBus::new());
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let mut handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));
    let state = build_state();

    // Act
    handler.process_sync_messages(peer_id, &state);

    // Assert
    assert!(log.sends.lock().expect("lock").is_empty());
}

#[test]
fn process_sync_messages_status_with_lag_sends_get_blocks_request() {
    // Arrange
    let peer_id = 1;
    let remote_peer = 2;
    let sync_bus = Arc::new(SyncBus::new());
    sync_bus.enqueue_sync_for(
        peer_id,
        remote_peer,
        SyncMessage::Status {
            height: 5,
            last_hash: [0u8; 32],
        },
    );
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let mut handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));
    let state = build_state();

    // Act
    handler.process_sync_messages(peer_id, &state);

    // Assert
    let sends = log.sends.lock().expect("lock");
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0, remote_peer);
    match &sends[0].1 {
        SyncMessage::GetBlocks {
            from_height,
            max_blocks,
        } => {
            assert_eq!(*from_height, 0);
            assert!(*max_blocks > 0);
        }
        other => panic!("expected GetBlocks, got {:?}", other),
    }
}

#[test]
fn process_sync_messages_status_without_lag_does_not_send_request() {
    // Arrange
    let peer_id = 1;
    let remote_peer = 2;
    let sync_bus = Arc::new(SyncBus::new());
    sync_bus.enqueue_sync_for(
        peer_id,
        remote_peer,
        SyncMessage::Status {
            height: 0,
            last_hash: [0u8; 32],
        },
    );
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let mut handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));
    let state = build_state();

    // Act
    handler.process_sync_messages(peer_id, &state);

    // Assert
    assert!(log.sends.lock().expect("lock").is_empty());
}

#[test]
fn process_sync_messages_get_blocks_responds_with_blocks_message() {
    // Arrange
    let peer_id = 1;
    let requesting_peer = 2;
    let block = Block::empty(0, peer_id, [0u8; 32]);
    let state = build_state_with_blocks(&[block]);
    let sync_bus = Arc::new(SyncBus::new());
    sync_bus.enqueue_sync_for(
        peer_id,
        requesting_peer,
        SyncMessage::GetBlocks {
            from_height: 0,
            max_blocks: 10,
        },
    );
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let mut handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));

    // Act
    handler.process_sync_messages(peer_id, &state);

    // Assert
    let sends = log.sends.lock().expect("lock");
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0, requesting_peer);
    match &sends[0].1 {
        SyncMessage::Blocks {
            start_height,
            block_payloads,
        } => {
            assert_eq!(*start_height, 0);
            assert_eq!(block_payloads.len(), 1);
        }
        other => panic!("expected Blocks, got {:?}", other),
    }
}

#[test]
fn process_sync_messages_get_blocks_empty_state_responds_with_empty_blocks() {
    // Arrange
    let peer_id = 1;
    let requesting_peer = 2;
    let sync_bus = Arc::new(SyncBus::new());
    sync_bus.enqueue_sync_for(
        peer_id,
        requesting_peer,
        SyncMessage::GetBlocks {
            from_height: 0,
            max_blocks: 10,
        },
    );
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let mut handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));
    let state = build_state();

    // Act
    handler.process_sync_messages(peer_id, &state);

    // Assert
    let sends = log.sends.lock().expect("lock");
    assert_eq!(sends.len(), 1);
    match &sends[0].1 {
        SyncMessage::Blocks {
            start_height,
            block_payloads,
        } => {
            assert_eq!(*start_height, 0);
            assert!(block_payloads.is_empty());
        }
        other => panic!("expected Blocks, got {:?}", other),
    }
}

#[test]
fn broadcast_status_delegates_to_sync_sender() {
    // Arrange
    let sync_bus = Arc::new(SyncBus::new());
    let log = FakeSyncLog::default();
    let sender = Box::new(FakeSyncSender::new(log.clone()));
    let sync_storage = TestInMemoryStorage::new().expect("failed to create sync storage");
    let handler = SyncHandler::new(sync_bus, sender, Box::new(sync_storage));

    // Act
    handler.broadcast_status(10, [7u8; 32]);

    // Assert
    let broadcasts = log.broadcasts.lock().expect("lock");
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].0, 10);
    assert_eq!(broadcasts[0].1, [7u8; 32]);
}
