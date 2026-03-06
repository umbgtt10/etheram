// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_bus::GrpcTransportBus;
use crate::infra::transport::grpc_transport::sync_bus::SyncBus;
use crate::infra::transport::grpc_transport::wire_node_message::deserialize;
use crate::infra::transport::grpc_transport::wire_node_message::NodeIncomingMessage;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use std::sync::Arc;

pub struct GrpcTransportIncoming {
    bus: Arc<GrpcTransportBus>,
    node_id: PeerId,
    sync_bus: Arc<SyncBus>,
}

impl GrpcTransportIncoming {
    pub fn new(
        node_id: PeerId,
        listen_addr: String,
        bus: Arc<GrpcTransportBus>,
        sync_bus: Arc<SyncBus>,
    ) -> Result<Self, String> {
        bus.ensure_server_started(node_id, &listen_addr)?;
        Ok(Self {
            bus,
            node_id,
            sync_bus,
        })
    }
}

impl TransportIncoming for GrpcTransportIncoming {
    type Message = IbftMessage;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        let (peer_id, payload) = self.bus.dequeue_for(self.node_id)?;
        match deserialize(&payload) {
            Ok(NodeIncomingMessage::Ibft(message)) => Some((peer_id, message)),
            Ok(NodeIncomingMessage::Sync(message)) => {
                self.sync_bus
                    .enqueue_sync_for(self.node_id, peer_id, message);
                None
            }
            Err(error) => {
                println!(
                    "grpc_receive_decode_error node_id={} from_peer={} error={}",
                    self.node_id, peer_id, error
                );
                None
            }
        }
    }
}
