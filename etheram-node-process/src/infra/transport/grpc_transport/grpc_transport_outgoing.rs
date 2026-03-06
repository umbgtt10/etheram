// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_bus::enqueue_to_local;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_client::TransportServiceClient;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportEnvelope;
use crate::infra::transport::grpc_transport::wire_node_message::serialize_ibft;
use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use std::collections::BTreeMap;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

const SEND_RETRY_COUNT: usize = 3;
const SEND_RETRY_INTERVAL_MS: u64 = 30;

pub struct GrpcTransportOutgoing {
    node_id: PeerId,
    peer_addresses: BTreeMap<PeerId, String>,
}

impl GrpcTransportOutgoing {
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
                .expect("failed building grpc runtime")
        })
    }

    fn send_with_retry(&self, peer_id: PeerId, address: &str, payload: &[u8]) {
        let mut last_error: Option<String> = None;
        for attempt in 1..=SEND_RETRY_COUNT {
            match Self::send_once(address, self.node_id, payload) {
                Ok(()) => {
                    return;
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }

            if attempt < SEND_RETRY_COUNT {
                thread::sleep(Duration::from_millis(SEND_RETRY_INTERVAL_MS));
            }
        }

        let error_message = last_error.unwrap_or_else(|| "unknown_error".to_string());
        println!(
            "grpc_send_error from_peer={} to_peer={} error={}",
            self.node_id, peer_id, error_message
        );
    }

    fn send_once(address: &str, from_peer: PeerId, payload: &[u8]) -> Result<(), String> {
        let payload_bytes = payload.to_vec();
        let endpoint = format!("http://{}", address);

        Self::runtime().block_on(async move {
            let mut client = TransportServiceClient::connect(endpoint)
                .await
                .map_err(|error| format!("failed connecting grpc client: {error}"))?;
            client
                .send_envelope(TransportEnvelope {
                    from_peer_id: from_peer,
                    ibft_message: payload_bytes,
                })
                .await
                .map_err(|error| format!("failed sending grpc envelope: {error}"))?;
            Ok(())
        })
    }
}

impl TransportOutgoing for GrpcTransportOutgoing {
    type Message = IbftMessage;

    fn send(&self, peer_id: PeerId, message: Self::Message) {
        let payload = match serialize_ibft(&message) {
            Ok(bytes) => bytes,
            Err(error) => {
                println!(
                    "grpc_send_error from_peer={} to_peer={} reason=encode_failed error={}",
                    self.node_id, peer_id, error
                );
                return;
            }
        };

        if peer_id == self.node_id {
            enqueue_to_local(self.node_id, self.node_id, payload);
            return;
        }

        if global_partition_table().is_blocked(self.node_id, peer_id) {
            println!(
                "partition_drop from_peer={} to_peer={}",
                self.node_id, peer_id
            );
            return;
        }

        let Some(address) = self.peer_addresses.get(&peer_id) else {
            println!(
                "grpc_send_error from_peer={} to_peer={} reason=unknown_peer",
                self.node_id, peer_id
            );
            return;
        };

        self.send_with_retry(peer_id, address, &payload);
    }
}
