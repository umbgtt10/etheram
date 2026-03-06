// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::transport::grpc_transport::wire_ibft_message::deserialize as deserialize_ibft;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize as serialize_ibft_wire;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use postcard::Error;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireNodeMessage {
    Ibft(Vec<u8>),
    Sync(SyncMessage),
}

pub enum NodeIncomingMessage {
    Ibft(IbftMessage),
    Sync(SyncMessage),
}

pub fn serialize_ibft(message: &IbftMessage) -> Result<Vec<u8>, Error> {
    let ibft_payload = serialize_ibft_wire(message)?;
    postcard::to_allocvec(&WireNodeMessage::Ibft(ibft_payload))
}

pub fn serialize_sync(message: &SyncMessage) -> Result<Vec<u8>, Error> {
    postcard::to_allocvec(&WireNodeMessage::Sync(message.clone()))
}

pub fn deserialize(bytes: &[u8]) -> Result<NodeIncomingMessage, Error> {
    let wire: WireNodeMessage = postcard::from_bytes(bytes)?;
    match wire {
        WireNodeMessage::Ibft(payload) => {
            Ok(NodeIncomingMessage::Ibft(deserialize_ibft(&payload)?))
        }
        WireNodeMessage::Sync(message) => Ok(NodeIncomingMessage::Sync(message)),
    }
}
