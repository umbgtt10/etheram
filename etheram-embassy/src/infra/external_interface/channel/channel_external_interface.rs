// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::channel::client_request_hub::CLIENT_REQUEST_HUB;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

embassy_core::define_channel_external_interface!(ClientRequest, ClientResponse, CLIENT_REQUEST_HUB);
