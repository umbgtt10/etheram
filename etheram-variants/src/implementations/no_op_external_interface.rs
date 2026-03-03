// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_core::external_interface_incoming::ExternalInterfaceIncoming;
use barechain_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use barechain_core::types::ClientId;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::incoming::external_interface::client_request::ClientRequest;

#[derive(Clone)]
pub struct NoOpExternalInterface;

impl ExternalInterfaceIncoming for NoOpExternalInterface {
    type Request = ClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        None
    }
}

impl ExternalInterfaceOutgoing for NoOpExternalInterface {
    type Response = ClientResponse;

    fn send_response(&self, _client_id: ClientId, _response: Self::Response) {}
}
