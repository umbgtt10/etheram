// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;
use etheram_node_process::infra::sync::sync_message::SyncMessage;
use etheram_node_process::infra::sync::sync_sender::GrpcSyncSender;
use etheram_node_process::infra::sync::sync_sender::SyncSender;
use etheram_node_process::infra::transport::grpc_transport::grpc_transport_bus::GrpcTransportBus;
use etheram_node_process::infra::transport::grpc_transport::grpc_transport_incoming::GrpcTransportIncoming;
use etheram_node_process::infra::transport::grpc_transport::sync_bus::SyncBus;
use etheram_node_process::infra::transport::partitionable_transport::partition_table::PartitionTable;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to allocate local port");
    listener
        .local_addr()
        .expect("failed to get local socket address")
        .port()
}

fn wait_for_sync_message(
    incoming: &GrpcTransportIncoming,
    sync_bus: &SyncBus,
    node_id: PeerId,
    retries: usize,
) -> Option<(PeerId, SyncMessage)> {
    for _ in 0..retries {
        let _ = incoming.poll();
        if let Some(message) = sync_bus.dequeue_sync_for(node_id) {
            return Some(message);
        }
        thread::sleep(Duration::from_millis(10));
    }
    None
}

fn send_and_wait_for_sync_message(
    sender: &GrpcSyncSender,
    incoming: &GrpcTransportIncoming,
    sync_bus: &SyncBus,
    to_peer: PeerId,
    message: &SyncMessage,
    attempts: usize,
) -> Option<(PeerId, SyncMessage)> {
    for _ in 0..attempts {
        sender.send_to_peer(to_peer, message);
        if let Some(queued) = wait_for_sync_message(incoming, sync_bus, to_peer, 6) {
            return Some(queued);
        }
    }
    None
}

#[test]
fn send_to_peer_unblocked_routes_sync_message_to_sync_queue() {
    // Arrange
    let from_peer = 41;
    let to_peer = 42;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming = GrpcTransportIncoming::new(to_peer, listen_addr.clone(), bus, sync_bus.clone())
        .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table);
    let message = SyncMessage::GetBlocks {
        from_height: 7,
        max_blocks: 16,
    };

    // Act
    thread::sleep(Duration::from_millis(40));
    let queued =
        send_and_wait_for_sync_message(&sender, &incoming, &sync_bus, to_peer, &message, 20);

    // Assert
    assert!(queued.is_some());
    let (queued_from, queued_message) = queued.expect("expected sync message");
    assert_eq!(queued_from, from_peer);
    assert_eq!(queued_message, message);
}

#[test]
fn send_to_peer_partitioned_then_healed_delivers_only_after_heal() {
    // Arrange
    let from_peer = 43;
    let to_peer = 44;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming = GrpcTransportIncoming::new(to_peer, listen_addr.clone(), bus, sync_bus.clone())
        .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table.clone());
    let message = SyncMessage::GetBlocks {
        from_height: 9,
        max_blocks: 8,
    };
    partition_table.block(from_peer, to_peer);

    // Act
    thread::sleep(Duration::from_millis(40));
    sender.send_to_peer(to_peer, &message);
    let blocked_observed = wait_for_sync_message(&incoming, &sync_bus, to_peer, 10);
    partition_table.heal(from_peer, to_peer);
    let healed_observed =
        send_and_wait_for_sync_message(&sender, &incoming, &sync_bus, to_peer, &message, 20);

    // Assert
    assert!(blocked_observed.is_none());
    assert!(healed_observed.is_some());
    let (queued_from, queued_message) = healed_observed.expect("expected healed sync message");
    assert_eq!(queued_from, from_peer);
    assert_eq!(queued_message, message);
}

#[test]
fn broadcast_status_two_peers_routes_status_to_each_peer_sync_queue() {
    // Arrange
    let from_peer = 45;
    let to_peer_1 = 46;
    let to_peer_2 = 47;
    let listen_addr_1 = format!("127.0.0.1:{}", next_port());
    let listen_addr_2 = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming_1 = GrpcTransportIncoming::new(
        to_peer_1,
        listen_addr_1.clone(),
        bus.clone(),
        sync_bus.clone(),
    )
    .expect("failed to create incoming 1");
    let incoming_2 =
        GrpcTransportIncoming::new(to_peer_2, listen_addr_2.clone(), bus, sync_bus.clone())
            .expect("failed to create incoming 2");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer_1, listen_addr_1);
    peer_addresses.insert(to_peer_2, listen_addr_2);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table);

    // Act
    thread::sleep(Duration::from_millis(40));
    sender.broadcast_status(21, [4u8; 32]);
    let queued_1 = wait_for_sync_message(&incoming_1, &sync_bus, to_peer_1, 20);
    let queued_2 = wait_for_sync_message(&incoming_2, &sync_bus, to_peer_2, 20);

    // Assert
    assert!(queued_1.is_some());
    assert!(queued_2.is_some());

    let (queued_from_1, queued_message_1) = queued_1.expect("expected sync message for peer 1");
    assert_eq!(queued_from_1, from_peer);
    assert_eq!(
        queued_message_1,
        SyncMessage::Status {
            height: 21,
            last_hash: [4u8; 32],
        }
    );

    let (queued_from_2, queued_message_2) = queued_2.expect("expected sync message for peer 2");
    assert_eq!(queued_from_2, from_peer);
    assert_eq!(
        queued_message_2,
        SyncMessage::Status {
            height: 21,
            last_hash: [4u8; 32],
        }
    );
}

#[test]
fn broadcast_status_with_self_in_peer_map_does_not_enqueue_to_self() {
    // Arrange
    let from_peer = 48;
    let to_peer = 49;
    let listen_addr_self = format!("127.0.0.1:{}", next_port());
    let listen_addr_peer = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming_self = GrpcTransportIncoming::new(
        from_peer,
        listen_addr_self.clone(),
        bus.clone(),
        sync_bus.clone(),
    )
    .expect("failed to create incoming self");
    let incoming_peer =
        GrpcTransportIncoming::new(to_peer, listen_addr_peer.clone(), bus, sync_bus.clone())
            .expect("failed to create incoming peer");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(from_peer, listen_addr_self);
    peer_addresses.insert(to_peer, listen_addr_peer);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table);

    // Act
    thread::sleep(Duration::from_millis(40));
    sender.broadcast_status(22, [5u8; 32]);
    let queued_self = wait_for_sync_message(&incoming_self, &sync_bus, from_peer, 10);
    let queued_peer = wait_for_sync_message(&incoming_peer, &sync_bus, to_peer, 20);

    // Assert
    assert!(queued_self.is_none());
    assert!(queued_peer.is_some());
}

#[test]
fn send_to_unknown_peer_does_not_enqueue_sync_message() {
    // Arrange
    let from_peer = 50;
    let existing_peer = 51;
    let unknown_peer = 52;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming =
        GrpcTransportIncoming::new(existing_peer, listen_addr.clone(), bus, sync_bus.clone())
            .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(existing_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table);
    let message = SyncMessage::GetBlocks {
        from_height: 30,
        max_blocks: 4,
    };

    // Act
    thread::sleep(Duration::from_millis(40));
    sender.send_to_peer(unknown_peer, &message);
    let queued = wait_for_sync_message(&incoming, &sync_bus, existing_peer, 10);

    // Assert
    assert!(queued.is_none());
}

#[test]
fn send_to_peer_long_partition_then_heal_multiple_attempts_deliver_after_heal() {
    // Arrange
    let from_peer = 53;
    let to_peer = 54;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let bus = Arc::new(GrpcTransportBus::new());
    let sync_bus = Arc::new(SyncBus::new());
    let partition_table = Arc::new(PartitionTable::new());
    let incoming = GrpcTransportIncoming::new(to_peer, listen_addr.clone(), bus, sync_bus.clone())
        .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses, partition_table.clone());
    let message = SyncMessage::GetBlocks {
        from_height: 40,
        max_blocks: 8,
    };
    partition_table.block(from_peer, to_peer);

    // Act
    thread::sleep(Duration::from_millis(40));
    for _ in 0..6 {
        sender.send_to_peer(to_peer, &message);
        thread::sleep(Duration::from_millis(10));
    }
    let blocked_observed = wait_for_sync_message(&incoming, &sync_bus, to_peer, 10);
    partition_table.heal(from_peer, to_peer);
    let healed_observed =
        send_and_wait_for_sync_message(&sender, &incoming, &sync_bus, to_peer, &message, 20);

    // Assert
    assert!(blocked_observed.is_none());
    assert!(healed_observed.is_some());
    let (queued_from, queued_message) = healed_observed.expect("expected message after heal");
    assert_eq!(queued_from, from_peer);
    assert_eq!(queued_message, message);
}
