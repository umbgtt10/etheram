// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::infra::external_interface::client_facade::await_ei_response;
use crate::infra::external_interface::client_facade::submit_ei_request;
use crate::infra::external_interface::client_facade::submit_ei_to_all_nodes;
use crate::spawned_node::SpawnedNode;
use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Timer;
use etheram::common_types::types::Height;
use etheram::common_types::types::{Address, Hash};
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_core::types::ClientId;
use etheram_etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;

pub struct EtheramClient {
    cancel: &'static CancellationToken,
    nodes: [SpawnedNode; MAX_NODES],
}

impl EtheramClient {
    pub fn new(cancel: &'static CancellationToken, nodes: [SpawnedNode; MAX_NODES]) -> Self {
        Self { cancel, nodes }
    }

    pub async fn fire_timer_all(&self, event: TimerEvent) {
        for node_index in 0..MAX_NODES {
            self.nodes[node_index].timer_sender.send(event).await;
        }
    }

    pub fn node_height(&self, node_index: usize) -> Height {
        self.nodes[node_index].read_height()
    }

    pub fn node_contract_storage(
        &self,
        node_index: usize,
        address: Address,
        slot: Hash,
    ) -> Option<Hash> {
        self.nodes[node_index].read_contract_storage(address, slot)
    }

    pub fn node_wal(&self, node_index: usize) -> Option<ConsensusWal> {
        self.nodes[node_index].read_wal()
    }

    pub fn node_last_cert(&self, node_index: usize) -> Option<PreparedCertificate> {
        self.nodes[node_index].read_last_cert()
    }

    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    pub fn submit_request(&self, node_index: usize, client_id: ClientId, request: ClientRequest) {
        submit_ei_request(node_index, client_id, request);
    }

    pub fn submit_to_all_nodes(&self, client_id: ClientId, request: ClientRequest) {
        submit_ei_to_all_nodes(client_id, request);
    }

    pub async fn await_response(&self, node_index: usize) -> (ClientId, ClientResponse) {
        await_ei_response(node_index).await
    }

    pub async fn wait_for_height_above(
        &self,
        node_index: usize,
        threshold: Height,
        timeout: Duration,
    ) {
        let _ = with_timeout(timeout, async {
            loop {
                if self.nodes[node_index].read_height() > threshold {
                    break;
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        })
        .await;
    }
}
