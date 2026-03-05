// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use etheram_core::types::ClientId;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

const REQUEST_CAPACITY: usize = 16;
const RESPONSE_CAPACITY: usize = 16;

type RequestChannel = Channel<CriticalSectionRawMutex, (ClientId, ClientRequest), REQUEST_CAPACITY>;

type ResponseChannel =
    Channel<CriticalSectionRawMutex, (ClientId, ClientResponse), RESPONSE_CAPACITY>;

type EiNotifyChannel = Channel<CriticalSectionRawMutex, (), 1>;

pub type EiNotifyReceiver = Receiver<'static, CriticalSectionRawMutex, (), 1>;

static EI_NOTIFY_CHANNELS: [EiNotifyChannel; MAX_NODES] = [
    Channel::new(),
    Channel::new(),
    Channel::new(),
    Channel::new(),
    Channel::new(),
];

pub fn ei_notify_receiver(node_index: usize) -> EiNotifyReceiver {
    EI_NOTIFY_CHANNELS[node_index].receiver()
}

pub struct ClientRequestHub {
    requests: [RequestChannel; MAX_NODES],
    responses: [ResponseChannel; MAX_NODES],
}

impl ClientRequestHub {
    pub const fn new() -> Self {
        Self {
            requests: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
            responses: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
        }
    }

    pub fn try_receive_request(&self, node_index: usize) -> Option<(ClientId, ClientRequest)> {
        self.requests[node_index].try_receive().ok()
    }

    pub fn try_send_response(
        &self,
        node_index: usize,
        client_id: ClientId,
        response: ClientResponse,
    ) {
        let _ = self.responses[node_index].try_send((client_id, response));
    }

    pub fn send_request(&self, node_index: usize, client_id: ClientId, request: ClientRequest) {
        let _ = self.requests[node_index].try_send((client_id, request));
        let _ = EI_NOTIFY_CHANNELS[node_index].try_send(());
    }

    pub async fn receive_response(&self, node_index: usize) -> (ClientId, ClientResponse) {
        self.responses[node_index].receive().await
    }
}

pub static CLIENT_REQUEST_HUB: ClientRequestHub = ClientRequestHub::new();
