// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::sync::sync_sender::GrpcSyncSender;
use crate::infra::sync::sync_sender::SyncSender;
use crate::infra::transport::grpc_transport::grpc_transport_incoming::GrpcTransportIncoming;
use crate::infra::transport::grpc_transport::sync_bus::dequeue_sync_for;
use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;
use std::collections::BTreeMap;
use std::net::TcpListener;
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
    node_id: PeerId,
    retries: usize,
) -> Option<(PeerId, SyncMessage)> {
    for _ in 0..retries {
        let _ = incoming.poll();
        if let Some(message) = dequeue_sync_for(node_id) {
            return Some(message);
        }
        thread::sleep(Duration::from_millis(10));
    }
    None
}

fn send_and_wait_for_sync_message(
    sender: &GrpcSyncSender,
    incoming: &GrpcTransportIncoming,
    to_peer: PeerId,
    message: &SyncMessage,
    attempts: usize,
) -> Option<(PeerId, SyncMessage)> {
    for _ in 0..attempts {
        sender.send_to_peer(to_peer, message);
        if let Some(queued) = wait_for_sync_message(incoming, to_peer, 6) {
            return Some(queued);
        }
    }
    None
}

#[test]
fn send_to_peer_unblocked_routes_sync_message_to_sync_queue() {
    // Arrange
    global_partition_table().clear();
    let from_peer = 41;
    let to_peer = 42;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming = GrpcTransportIncoming::new(to_peer, listen_addr.clone())
        .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses);
    let message = SyncMessage::GetBlocks {
        from_height: 7,
        max_blocks: 16,
    };

    // Act
    thread::sleep(Duration::from_millis(40));
    let queued = send_and_wait_for_sync_message(&sender, &incoming, to_peer, &message, 20);

    // Assert
    assert!(queued.is_some());
    let (queued_from, queued_message) = queued.expect("expected sync message");
    assert_eq!(queued_from, from_peer);
    assert_eq!(queued_message, message);
}

#[test]
fn send_to_peer_partitioned_then_healed_delivers_only_after_heal() {
    // Arrange
    global_partition_table().clear();
    let from_peer = 43;
    let to_peer = 44;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming = GrpcTransportIncoming::new(to_peer, listen_addr.clone())
        .expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, listen_addr);
    let sender = GrpcSyncSender::new(from_peer, peer_addresses);
    let message = SyncMessage::GetBlocks {
        from_height: 9,
        max_blocks: 8,
    };
    global_partition_table().block(from_peer, to_peer);

    // Act
    thread::sleep(Duration::from_millis(40));
    sender.send_to_peer(to_peer, &message);
    let blocked_observed = wait_for_sync_message(&incoming, to_peer, 10);
    global_partition_table().heal(from_peer, to_peer);
    let healed_observed = send_and_wait_for_sync_message(&sender, &incoming, to_peer, &message, 20);

    // Assert
    assert!(blocked_observed.is_none());
    assert!(healed_observed.is_some());
    let (queued_from, queued_message) = healed_observed.expect("expected healed sync message");
    assert_eq!(queued_from, from_peer);
    assert_eq!(queued_message, message);
}
