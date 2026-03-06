// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_client::TransportServiceClient;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportEnvelope;
use crate::infra::transport::grpc_transport::wire_node_message::serialize_sync;
use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use crate::infra::transport::transport_backend::TransportBackend;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub trait SyncSender {
    fn broadcast_status(&self, height: Height, last_hash: Hash);
}

pub struct GrpcSyncSender {
    node_id: PeerId,
    peer_addresses: BTreeMap<PeerId, String>,
}

impl GrpcSyncSender {
    pub fn new(node_id: PeerId, peer_addresses: BTreeMap<PeerId, String>) -> Self {
        Self {
            node_id,
            peer_addresses,
        }
    }

    fn runtime() -> &'static tokio::runtime::Runtime {
        static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(1)
                .build()
                .expect("failed building sync grpc runtime")
        })
    }

    fn send_sync(&self, peer_id: PeerId, message: &SyncMessage) {
        if global_partition_table().is_blocked(self.node_id, peer_id) {
            return;
        }

        let Some(address) = self.peer_addresses.get(&peer_id) else {
            return;
        };

        let Ok(payload) = serialize_sync(message) else {
            return;
        };

        let endpoint = format!("http://{}", address);
        let from_peer = self.node_id;

        let _ = Self::runtime().block_on(async move {
            let mut client = TransportServiceClient::connect(endpoint)
                .await
                .map_err(|_| ())?;
            client
                .send_envelope(TransportEnvelope {
                    from_peer_id: from_peer,
                    ibft_message: payload,
                })
                .await
                .map_err(|_| ())?;
            Ok::<(), ()>(())
        });
    }
}

impl SyncSender for GrpcSyncSender {
    fn broadcast_status(&self, height: Height, last_hash: Hash) {
        let message = SyncMessage::Status { height, last_hash };
        for peer_id in self.peer_addresses.keys().copied() {
            if peer_id == self.node_id {
                continue;
            }
            self.send_sync(peer_id, &message);
        }
    }
}

pub struct NoOpSyncSender;

impl SyncSender for NoOpSyncSender {
    fn broadcast_status(&self, _height: Height, _last_hash: Hash) {}
}

pub fn build_sync_sender(
    backend: &TransportBackend,
    node_id: PeerId,
    peer_addresses: &BTreeMap<PeerId, String>,
) -> Box<dyn SyncSender> {
    match backend {
        TransportBackend::LocalNoOp => Box::new(NoOpSyncSender),
        TransportBackend::Grpc => Box::new(GrpcSyncSender::new(node_id, peer_addresses.clone())),
    }
}
