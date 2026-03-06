// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_client_message::wire_client_request::WireClientRequest;
use crate::infra::external_interface::udp::wire_client_message::wire_client_response::WireClientResponse;
use crate::infra::external_interface::udp::wire_client_message::wire_ei_request::WireEiRequest;
use crate::infra::external_interface::udp::wire_client_message::wire_ei_response::WireEiResponse;
use alloc::vec::Vec;
use etheram_core::types::ClientId;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

pub fn serialize_ei_request(client_id: ClientId, request: ClientRequest) -> Vec<u8> {
    let wire = WireEiRequest {
        client_id,
        request: WireClientRequest::from(request),
    };
    postcard::to_allocvec(&wire).unwrap_or_default()
}

pub fn deserialize_ei_request(bytes: &[u8]) -> Option<(ClientId, ClientRequest)> {
    postcard::from_bytes::<WireEiRequest>(bytes)
        .ok()
        .map(|wire| (wire.client_id, ClientRequest::from(wire.request)))
}

pub fn serialize_ei_response(client_id: ClientId, response: ClientResponse) -> Vec<u8> {
    let wire = WireEiResponse {
        client_id,
        response: WireClientResponse::from(response),
    };
    postcard::to_allocvec(&wire).unwrap_or_default()
}

pub fn deserialize_ei_response(bytes: &[u8]) -> Option<(ClientId, ClientResponse)> {
    postcard::from_bytes::<WireEiResponse>(bytes)
        .ok()
        .map(|wire| (wire.client_id, ClientResponse::from(wire.response)))
}
