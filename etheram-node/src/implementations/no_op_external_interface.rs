// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::executor::outgoing::external_interface::client_response::ClientResponse;
use crate::incoming::external_interface::client_request::ClientRequest;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::types::ClientId;

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
