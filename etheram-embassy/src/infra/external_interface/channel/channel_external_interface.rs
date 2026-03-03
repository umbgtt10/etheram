// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::channel::client_request_hub::CLIENT_REQUEST_HUB;
use barechain_core::external_interface_incoming::ExternalInterfaceIncoming;
use barechain_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use barechain_core::types::ClientId;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::incoming::external_interface::client_request::ClientRequest;

pub struct ChannelExternalInterface {
    node_index: usize,
}

impl ChannelExternalInterface {
    pub fn new(node_index: usize) -> Self {
        Self { node_index }
    }
}

impl ExternalInterfaceIncoming for ChannelExternalInterface {
    type Request = ClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        CLIENT_REQUEST_HUB.try_receive_request(self.node_index)
    }
}

impl ExternalInterfaceOutgoing for ChannelExternalInterface {
    type Response = ClientResponse;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        CLIENT_REQUEST_HUB.try_send_response(self.node_index, client_id, response);
    }
}
