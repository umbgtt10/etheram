// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_raft_client_message::wire_ei_request_packet::WireEiRequestPacket;
use crate::infra::external_interface::udp::wire_raft_client_message::wire_ei_response_packet::WireEiResponsePacket;
use crate::infra::external_interface::udp::wire_raft_client_message::wire_raft_client_request::WireRaftClientRequest;
use crate::infra::external_interface::udp::wire_raft_client_message::wire_raft_client_response::WireRaftClientResponse;
use alloc::vec::Vec;
use etheram_core::types::ClientId;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;

pub fn serialize_ei_request(client_id: ClientId, request: RaftClientRequest) -> Vec<u8> {
    let packet = WireEiRequestPacket {
        client_id,
        payload: WireRaftClientRequest::from(request),
    };
    postcard::to_allocvec(&packet).unwrap_or_default()
}

pub fn deserialize_ei_request(bytes: &[u8]) -> Option<(ClientId, RaftClientRequest)> {
    let packet: WireEiRequestPacket = postcard::from_bytes(bytes).ok()?;
    Some((packet.client_id, packet.payload.into()))
}

pub fn serialize_ei_response(client_id: ClientId, response: RaftClientResponse) -> Vec<u8> {
    let packet = WireEiResponsePacket {
        client_id,
        payload: WireRaftClientResponse::from(response),
    };
    postcard::to_allocvec(&packet).unwrap_or_default()
}

pub fn deserialize_ei_response(bytes: &[u8]) -> Option<(ClientId, RaftClientResponse)> {
    let packet: WireEiResponsePacket = postcard::from_bytes(bytes).ok()?;
    Some((packet.client_id, packet.payload.into()))
}
