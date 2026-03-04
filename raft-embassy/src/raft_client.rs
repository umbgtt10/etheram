// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::infra::external_interface::client_facade::await_ei_response;
use crate::infra::external_interface::client_facade::submit_ei_request;
use crate::infra::external_interface::client_facade::submit_ei_to_all_nodes;
use crate::spawned_node::SpawnedNode;
use alloc::string::String;
use alloc::vec::Vec;
use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Timer;
use etheram_core::types::ClientId;
use raft_node::common_types::node_role::NodeRole;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

pub struct RaftClient {
    cancel: &'static CancellationToken,
    nodes: [SpawnedNode; MAX_NODES],
}

impl RaftClient {
    pub fn new(cancel: &'static CancellationToken, nodes: [SpawnedNode; MAX_NODES]) -> Self {
        Self { cancel, nodes }
    }

    pub async fn fire_timer_to(&self, node_index: usize, event: RaftTimerEvent) {
        self.nodes[node_index].timer_sender.send(event).await;
    }

    pub fn node_commit_index(&self, node_index: usize) -> u64 {
        self.nodes[node_index].read_commit_index()
    }

    pub fn node_term(&self, node_index: usize) -> u64 {
        self.nodes[node_index].read_term()
    }

    pub fn find_leader(&self) -> Option<usize> {
        (0..MAX_NODES).find(|&i| self.nodes[i].read_role() == NodeRole::Leader)
    }

    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    pub fn submit_command_all(&self, client_id: ClientId, payload: Vec<u8>) {
        submit_ei_to_all_nodes(client_id, RaftClientRequest::Command(payload));
    }

    pub fn submit_query(&self, node_index: usize, client_id: ClientId, key: String) {
        submit_ei_request(node_index, client_id, RaftClientRequest::Query(key));
    }

    pub async fn await_response(&self, node_index: usize) -> (ClientId, RaftClientResponse) {
        await_ei_response(node_index).await
    }

    pub async fn wait_for_commit_above(
        &self,
        node_index: usize,
        threshold: u64,
        timeout: Duration,
    ) {
        let _ = with_timeout(timeout, async {
            loop {
                if self.nodes[node_index].read_commit_index() > threshold {
                    break;
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        })
        .await;
    }

    pub async fn wait_for_term_above(&self, node_index: usize, threshold: u64, timeout: Duration) {
        let _ = with_timeout(timeout, async {
            loop {
                if self.nodes[node_index].read_term() > threshold {
                    break;
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        })
        .await;
    }

    pub async fn wait_for_leader(&self, timeout: Duration) -> Option<usize> {
        let result = with_timeout(timeout, async {
            loop {
                if let Some(leader) = self.find_leader() {
                    return leader;
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        })
        .await;
        result.ok()
    }
}
