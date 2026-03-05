// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use embassy_core::client_request_hub::GenericClientRequestHub;
use embassy_core::client_request_hub::GenericEiNotifyReceiver;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

const REQUEST_CAPACITY: usize = 16;
const RESPONSE_CAPACITY: usize = 16;

const NOTIFY_CAPACITY: usize = 1;

pub type EiNotifyReceiver = GenericEiNotifyReceiver<'static, NOTIFY_CAPACITY>;

type ClientRequestHub = GenericClientRequestHub<
    ClientRequest,
    ClientResponse,
    MAX_NODES,
    REQUEST_CAPACITY,
    RESPONSE_CAPACITY,
    NOTIFY_CAPACITY,
>;

pub fn ei_notify_receiver(node_index: usize) -> EiNotifyReceiver {
    CLIENT_REQUEST_HUB.notify_receiver(node_index)
}

pub static CLIENT_REQUEST_HUB: ClientRequestHub = ClientRequestHub::new();
