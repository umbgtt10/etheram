// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node_process::infra::sync::sync_message::SyncMessage;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::deserialize;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::serialize_ibft;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::serialize_sync;
use etheram_node_process::infra::transport::grpc_transport::wire_node_message::NodeIncomingMessage;

#[test]
fn serialize_ibft_then_deserialize_returns_ibft_message() {
    // Arrange
    let message = IbftMessage::ViewChange {
        sequence: 5,
        height: 2,
        round: 1,
        prepared_certificate: None,
    };

    // Act
    let bytes = serialize_ibft(&message).expect("failed to serialize ibft");
    let decoded = deserialize(&bytes).expect("failed to deserialize node message");

    // Assert
    match decoded {
        NodeIncomingMessage::Ibft(observed) => assert_eq!(observed, message),
        NodeIncomingMessage::Sync(_) => panic!("expected ibft message"),
    }
}

#[test]
fn serialize_sync_then_deserialize_returns_sync_message() {
    // Arrange
    let message = SyncMessage::Status {
        height: 10,
        last_hash: [9u8; 32],
    };

    // Act
    let bytes = serialize_sync(&message).expect("failed to serialize sync");
    let decoded = deserialize(&bytes).expect("failed to deserialize node message");

    // Assert
    match decoded {
        NodeIncomingMessage::Sync(observed) => assert_eq!(observed, message),
        NodeIncomingMessage::Ibft(_) => panic!("expected sync message"),
    }
}

#[test]
fn serialize_sync_get_blocks_then_deserialize_returns_get_blocks() {
    // Arrange
    let message = SyncMessage::GetBlocks {
        from_height: 17,
        max_blocks: 64,
    };

    // Act
    let bytes = serialize_sync(&message).expect("failed to serialize sync");
    let decoded = deserialize(&bytes).expect("failed to deserialize node message");

    // Assert
    match decoded {
        NodeIncomingMessage::Sync(observed) => assert_eq!(observed, message),
        NodeIncomingMessage::Ibft(_) => panic!("expected sync message"),
    }
}

#[test]
fn serialize_sync_blocks_then_deserialize_returns_blocks() {
    // Arrange
    let message = SyncMessage::Blocks {
        start_height: 42,
        block_payloads: vec![vec![1, 2, 3], vec![4, 5]],
    };

    // Act
    let bytes = serialize_sync(&message).expect("failed to serialize sync");
    let decoded = deserialize(&bytes).expect("failed to deserialize node message");

    // Assert
    match decoded {
        NodeIncomingMessage::Sync(observed) => assert_eq!(observed, message),
        NodeIncomingMessage::Ibft(_) => panic!("expected sync message"),
    }
}

#[test]
fn deserialize_invalid_wire_bytes_returns_error() {
    // Arrange
    let invalid_wire = vec![1u8, 2u8, 3u8, 4u8];

    // Act
    let decoded = deserialize(&invalid_wire);

    // Assert
    assert!(decoded.is_err());
}

#[test]
fn deserialize_ibft_wire_with_invalid_nested_payload_returns_error() {
    // Arrange
    let invalid_nested = postcard::to_allocvec(&(0u32, vec![7u8, 8u8, 9u8]))
        .expect("failed to build invalid nested payload");

    // Act
    let decoded = deserialize(&invalid_nested);

    // Assert
    assert!(decoded.is_err());
}
