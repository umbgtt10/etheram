// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node_process::infra::sync::sync_message::SyncMessage;
use etheram_node_process::infra::transport::grpc_transport::grpc_transport_bus::enqueue_to_local;
use etheram_node_process::infra::transport::grpc_transport::grpc_transport_incoming::GrpcTransportIncoming;
use etheram_node_process::infra::transport::grpc_transport::grpc_transport_outgoing::GrpcTransportOutgoing;
use etheram_node_process::infra::transport::grpc_transport::sync_bus::dequeue_sync_for;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::serialize_ibft;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::serialize_sync;
use etheram_node_process::infra::transport::partitionable_transport::partition_table::global_partition_table;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn sample_message() -> IbftMessage {
    sample_message_with_sequence(1)
}

fn sample_message_with_sequence(sequence: u64) -> IbftMessage {
    IbftMessage::ViewChange {
        sequence,
        height: 1,
        round: 0,
        prepared_certificate: None,
    }
}

fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to allocate local port");
    listener
        .local_addr()
        .expect("failed to get local socket address")
        .port()
}

fn wait_for_message(
    incoming: &GrpcTransportIncoming,
    retries: usize,
    sleep_ms: u64,
) -> Option<(PeerId, IbftMessage)> {
    for _ in 0..retries {
        if let Some(message) = incoming.poll() {
            return Some(message);
        }
        thread::sleep(Duration::from_millis(sleep_ms));
    }
    None
}

#[test]
fn poll_empty_queue_returns_none() {
    // Arrange
    let node_id = 11;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");

    // Act
    let result = incoming.poll();

    // Assert
    assert!(result.is_none());
}

#[test]
fn send_self_message_poll_returns_same_message() {
    // Arrange
    global_partition_table().clear();
    let node_id = 12;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    let outgoing = GrpcTransportOutgoing::new(node_id, BTreeMap::new());
    let expected = sample_message();

    // Act
    outgoing.send(node_id, expected.clone());
    let observed = wait_for_message(&incoming, 20, 10);

    // Assert
    assert!(observed.is_some());
    let (from_peer, message) = observed.expect("expected loopback message");
    assert_eq!(from_peer, node_id);
    assert_eq!(message, expected);
}

#[test]
fn send_unknown_peer_poll_returns_none() {
    // Arrange
    global_partition_table().clear();
    let node_id = 13;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    let outgoing = GrpcTransportOutgoing::new(node_id, BTreeMap::new());

    // Act
    outgoing.send(99, sample_message());
    let observed = wait_for_message(&incoming, 8, 10);

    // Assert
    assert!(observed.is_none());
}

#[test]
fn send_blocked_link_poll_returns_none() {
    // Arrange
    global_partition_table().clear();
    let from_peer = 14;
    let to_peer = 15;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(to_peer, listen_addr).expect("failed to create incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, "127.0.0.1:9".to_string());
    let outgoing = GrpcTransportOutgoing::new(from_peer, peer_addresses);
    global_partition_table().block(from_peer, to_peer);

    // Act
    outgoing.send(to_peer, sample_message());
    let observed = wait_for_message(&incoming, 8, 10);

    // Assert
    assert!(observed.is_none());
    global_partition_table().heal(from_peer, to_peer);
}

#[test]
fn send_remote_message_poll_on_receiver_returns_message() {
    // Arrange
    global_partition_table().clear();
    let from_peer = 16;
    let to_peer = 17;
    let receiver_addr = format!("127.0.0.1:{}", next_port());
    let incoming = GrpcTransportIncoming::new(to_peer, receiver_addr.clone())
        .expect("failed to create receiver incoming");
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(to_peer, receiver_addr);
    let outgoing = GrpcTransportOutgoing::new(from_peer, peer_addresses);
    let expected = sample_message();

    // Act
    thread::sleep(Duration::from_millis(40));
    outgoing.send(to_peer, expected.clone());
    let observed = wait_for_message(&incoming, 40, 10);

    // Assert
    assert!(observed.is_some());
    let (sender, message) = observed.expect("expected remote message");
    assert_eq!(sender, from_peer);
    assert_eq!(message, expected);
}

#[test]
fn poll_invalid_payload_then_valid_payload_returns_valid_message() {
    // Arrange
    let node_id = 18;
    let from_peer = 19;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    enqueue_to_local(node_id, from_peer, vec![1, 2, 3, 4]);
    let valid = sample_message();
    let payload = serialize_ibft(&valid).expect("failed to serialize valid message");
    enqueue_to_local(node_id, from_peer, payload);

    // Act
    let first = incoming.poll();
    let second = incoming.poll();

    // Assert
    assert!(first.is_none());
    assert!(second.is_some());
    let (sender, message) = second.expect("expected valid message after invalid payload");
    assert_eq!(sender, from_peer);
    assert_eq!(message, valid);
}

#[test]
fn poll_sync_payload_routes_to_sync_queue_returns_none() {
    // Arrange
    let node_id = 26;
    let from_peer = 27;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    let sync_message = SyncMessage::Status {
        height: 7,
        last_hash: [3u8; 32],
    };
    let payload = serialize_sync(&sync_message).expect("failed to serialize sync message");
    enqueue_to_local(node_id, from_peer, payload);

    // Act
    let observed = incoming.poll();
    let queued_sync = dequeue_sync_for(node_id);

    // Assert
    assert!(observed.is_none());
    assert!(queued_sync.is_some());
    let (queued_peer, queued_message) = queued_sync.expect("expected queued sync message");
    assert_eq!(queued_peer, from_peer);
    assert_eq!(queued_message, sync_message);
}

#[test]
fn poll_sync_get_blocks_payload_routes_to_sync_queue_returns_none() {
    // Arrange
    let node_id = 28;
    let from_peer = 29;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    let sync_message = SyncMessage::GetBlocks {
        from_height: 11,
        max_blocks: 32,
    };
    let payload = serialize_sync(&sync_message).expect("failed to serialize sync message");
    enqueue_to_local(node_id, from_peer, payload);

    // Act
    let observed = incoming.poll();
    let queued_sync = dequeue_sync_for(node_id);

    // Assert
    assert!(observed.is_none());
    assert!(queued_sync.is_some());
    let (queued_peer, queued_message) = queued_sync.expect("expected queued sync message");
    assert_eq!(queued_peer, from_peer);
    assert_eq!(queued_message, sync_message);
}

#[test]
fn poll_sync_blocks_payload_routes_to_sync_queue_returns_none() {
    // Arrange
    let node_id = 30;
    let from_peer = 31;
    let listen_addr = format!("127.0.0.1:{}", next_port());
    let incoming =
        GrpcTransportIncoming::new(node_id, listen_addr).expect("failed to create incoming");
    let sync_message = SyncMessage::Blocks {
        start_height: 13,
        block_payloads: vec![vec![1u8, 2u8], vec![3u8]],
    };
    let payload = serialize_sync(&sync_message).expect("failed to serialize sync message");
    enqueue_to_local(node_id, from_peer, payload);

    // Act
    let observed = incoming.poll();
    let queued_sync = dequeue_sync_for(node_id);

    // Assert
    assert!(observed.is_none());
    assert!(queued_sync.is_some());
    let (queued_peer, queued_message) = queued_sync.expect("expected queued sync message");
    assert_eq!(queued_peer, from_peer);
    assert_eq!(queued_message, sync_message);
}
