// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_client::TransportServiceClient;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportEnvelope;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize;
use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use std::collections::BTreeMap;
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

    fn send_with_retry(&self, peer_id: PeerId, address: &str, payload: &[u8]) {
        for attempt in 1..=SEND_RETRY_COUNT {
            if Self::send_once(address, self.node_id, payload).is_ok() {
                return;
            }
            if attempt < SEND_RETRY_COUNT {
                thread::sleep(Duration::from_millis(SEND_RETRY_INTERVAL_MS));
            }
        }
        println!(
            "grpc_send_error from_peer={} to_peer={} addr={}",
            self.node_id, peer_id, address
        );
    }

    fn send_once(address: &str, from_peer: PeerId, payload: &[u8]) -> Result<(), String> {
        let endpoint = format!("http://{}", address);
        let payload_bytes = payload.to_vec();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("failed building grpc runtime: {error}"))?;

        runtime.block_on(async move {
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
        if global_partition_table().is_blocked(self.node_id, peer_id) {
            println!(
                "partition_drop from_peer={} to_peer={}",
                self.node_id, peer_id
            );
            return;
        }
        let payload = match serialize(&message) {
            Ok(bytes) => bytes,
            Err(error) => {
                println!(
                    "grpc_send_error from_peer={} to_peer={} reason=encode_failed error={}",
                    self.node_id, peer_id, error
                );
                return;
            }
        };
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
