// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_client::TransportServiceClient;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportEnvelope;
use crate::infra::transport::grpc_transport::wire_node_message::serialize_sync;
use crate::infra::transport::partitionable_transport::partition_table::PartitionTable;
use crate::infra::transport::transport_backend::TransportBackend;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

const SYNC_SEND_RETRY_COUNT: usize = 3;
const SYNC_SEND_RETRY_INTERVAL_MS: u64 = 30;

pub trait SyncSender {
    fn broadcast_status(&self, height: Height, last_hash: Hash);
    fn send_to_peer(&self, peer_id: PeerId, message: &SyncMessage);
}

pub struct GrpcSyncSender {
    channel_cache: Mutex<BTreeMap<PeerId, Channel>>,
    node_id: PeerId,
    partition_table: Arc<PartitionTable>,
    peer_addresses: BTreeMap<PeerId, String>,
}

impl GrpcSyncSender {
    pub fn new(
        node_id: PeerId,
        peer_addresses: BTreeMap<PeerId, String>,
        partition_table: Arc<PartitionTable>,
    ) -> Self {
        Self {
            channel_cache: Mutex::new(BTreeMap::new()),
            node_id,
            partition_table,
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

    fn channel_for_peer(&self, peer_id: PeerId, address: &str) -> Option<Channel> {
        let mut guard = self.channel_cache.lock().ok()?;

        if let Some(channel) = guard.get(&peer_id) {
            return Some(channel.clone());
        }

        let endpoint = Endpoint::from_shared(format!("http://{}", address)).ok()?;
        let channel = Self::runtime()
            .block_on(async { endpoint.connect().await })
            .ok()?;
        guard.insert(peer_id, channel.clone());
        Some(channel)
    }

    fn invalidate_channel(&self, peer_id: PeerId) {
        if let Ok(mut guard) = self.channel_cache.lock() {
            guard.remove(&peer_id);
        }
    }

    fn send_sync(&self, peer_id: PeerId, message: &SyncMessage) {
        if self.partition_table.is_blocked(self.node_id, peer_id) {
            return;
        }

        let Some(address) = self.peer_addresses.get(&peer_id) else {
            return;
        };

        let Ok(payload) = serialize_sync(message) else {
            return;
        };

        let from_peer = self.node_id;
        for attempt in 1..=SYNC_SEND_RETRY_COUNT {
            let Some(channel) = self.channel_for_peer(peer_id, address) else {
                return;
            };
            let mut client = TransportServiceClient::new(channel);
            let send_result = Self::runtime().block_on(async {
                client
                    .send_envelope(TransportEnvelope {
                        from_peer_id: from_peer,
                        ibft_message: payload.clone(),
                    })
                    .await
                    .map_err(|_| ())
            });

            if send_result.is_ok() {
                return;
            }

            self.invalidate_channel(peer_id);
            if attempt < SYNC_SEND_RETRY_COUNT {
                thread::sleep(Duration::from_millis(SYNC_SEND_RETRY_INTERVAL_MS));
            }
        }
    }
}

impl SyncSender for GrpcSyncSender {
    fn broadcast_status(&self, height: Height, last_hash: Hash) {
        let message = SyncMessage::Status { height, last_hash };
        for peer_id in self.peer_addresses.keys().copied() {
            if peer_id == self.node_id {
                continue;
            }
            self.send_to_peer(peer_id, &message);
        }
    }

    fn send_to_peer(&self, peer_id: PeerId, message: &SyncMessage) {
        self.send_sync(peer_id, message);
    }
}

pub struct NoOpSyncSender;

impl SyncSender for NoOpSyncSender {
    fn broadcast_status(&self, _height: Height, _last_hash: Hash) {}

    fn send_to_peer(&self, _peer_id: PeerId, _message: &SyncMessage) {}
}

pub fn build_sync_sender(
    backend: &TransportBackend,
    node_id: PeerId,
    peer_addresses: &BTreeMap<PeerId, String>,
    partition_table: Arc<PartitionTable>,
) -> Box<dyn SyncSender> {
    match backend {
        TransportBackend::LocalNoOp => Box::new(NoOpSyncSender),
        TransportBackend::Grpc => Box::new(GrpcSyncSender::new(
            node_id,
            peer_addresses.clone(),
            partition_table,
        )),
    }
}
