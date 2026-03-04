// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
#[cfg(feature = "channel-external-interface")]
use crate::infra::external_interface::channel::client_request_hub::CLIENT_REQUEST_HUB;
#[cfg(feature = "udp-external-interface")]
use crate::infra::external_interface::udp::udp_raft_external_interface::receive_udp_raft_ei_response;
#[cfg(feature = "udp-external-interface")]
use crate::infra::external_interface::udp::udp_raft_external_interface::send_udp_raft_ei_request;
use etheram_core::types::ClientId;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;

pub fn submit_ei_request(node_index: usize, client_id: ClientId, request: RaftClientRequest) {
    submit_impl(node_index, client_id, request);
}

pub fn submit_ei_to_all_nodes(client_id: ClientId, request: RaftClientRequest) {
    for node_index in 0..MAX_NODES {
        submit_impl(node_index, client_id, request.clone());
    }
}

pub async fn await_ei_response(node_index: usize) -> (ClientId, RaftClientResponse) {
    await_impl(node_index).await
}

#[cfg(feature = "channel-external-interface")]
fn submit_impl(node_index: usize, client_id: ClientId, request: RaftClientRequest) {
    CLIENT_REQUEST_HUB.send_request(node_index, client_id, request);
}

#[cfg(feature = "channel-external-interface")]
async fn await_impl(node_index: usize) -> (ClientId, RaftClientResponse) {
    CLIENT_REQUEST_HUB.receive_response(node_index).await
}

#[cfg(feature = "udp-external-interface")]
fn submit_impl(node_index: usize, client_id: ClientId, request: RaftClientRequest) {
    send_udp_raft_ei_request(node_index, client_id, request);
}

#[cfg(feature = "udp-external-interface")]
async fn await_impl(node_index: usize) -> (ClientId, RaftClientResponse) {
    receive_udp_raft_ei_response(node_index).await
}
