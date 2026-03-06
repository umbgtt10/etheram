// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::transport::grpc_transport::wire_node_message::deserialize;
use crate::infra::transport::grpc_transport::wire_node_message::serialize_ibft;
use crate::infra::transport::grpc_transport::wire_node_message::serialize_sync;
use crate::infra::transport::grpc_transport::wire_node_message::NodeIncomingMessage;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;

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
