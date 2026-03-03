// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_client_message::deserialize_ei_request;
use crate::infra::external_interface::udp::wire_client_message::deserialize_ei_response;
use crate::infra::external_interface::udp::wire_client_message::serialize_ei_request;
use crate::infra::external_interface::udp::wire_client_message::serialize_ei_response;
use alloc::vec::Vec;
use barechain_core::external_interface_incoming::ExternalInterfaceIncoming;
use barechain_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use barechain_core::types::ClientId;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::incoming::external_interface::client_request::ClientRequest;

const EI_CAPACITY: usize = 16;

type EiPacketChannel = Channel<CriticalSectionRawMutex, Vec<u8>, EI_CAPACITY>;

type EiNotifyChannel = Channel<CriticalSectionRawMutex, (), 1>;

pub type UdpEiNotifyReceiver = Receiver<'static, CriticalSectionRawMutex, (), 1>;

static UDP_EI_NOTIFY_0: EiNotifyChannel = Channel::new();
static UDP_EI_NOTIFY_1: EiNotifyChannel = Channel::new();
static UDP_EI_NOTIFY_2: EiNotifyChannel = Channel::new();
static UDP_EI_NOTIFY_3: EiNotifyChannel = Channel::new();
static UDP_EI_NOTIFY_4: EiNotifyChannel = Channel::new();

pub fn udp_ei_notify_receiver(node_index: usize) -> UdpEiNotifyReceiver {
    match node_index {
        0 => UDP_EI_NOTIFY_0.receiver(),
        1 => UDP_EI_NOTIFY_1.receiver(),
        2 => UDP_EI_NOTIFY_2.receiver(),
        3 => UDP_EI_NOTIFY_3.receiver(),
        4 => UDP_EI_NOTIFY_4.receiver(),
        _ => panic!("invalid node_index"),
    }
}

static UDP_EI_REQUEST_0: EiPacketChannel = Channel::new();
static UDP_EI_REQUEST_1: EiPacketChannel = Channel::new();
static UDP_EI_REQUEST_2: EiPacketChannel = Channel::new();
static UDP_EI_REQUEST_3: EiPacketChannel = Channel::new();
static UDP_EI_REQUEST_4: EiPacketChannel = Channel::new();

static UDP_EI_RESPONSE_0: EiPacketChannel = Channel::new();
static UDP_EI_RESPONSE_1: EiPacketChannel = Channel::new();
static UDP_EI_RESPONSE_2: EiPacketChannel = Channel::new();
static UDP_EI_RESPONSE_3: EiPacketChannel = Channel::new();
static UDP_EI_RESPONSE_4: EiPacketChannel = Channel::new();

pub fn send_udp_ei_request(node_index: usize, client_id: ClientId, request: ClientRequest) {
    let bytes = serialize_ei_request(client_id, request);
    let sender = match node_index {
        0 => UDP_EI_REQUEST_0.sender(),
        1 => UDP_EI_REQUEST_1.sender(),
        2 => UDP_EI_REQUEST_2.sender(),
        3 => UDP_EI_REQUEST_3.sender(),
        4 => UDP_EI_REQUEST_4.sender(),
        _ => return,
    };
    let _ = sender.try_send(bytes);
    let notify = match node_index {
        0 => UDP_EI_NOTIFY_0.sender(),
        1 => UDP_EI_NOTIFY_1.sender(),
        2 => UDP_EI_NOTIFY_2.sender(),
        3 => UDP_EI_NOTIFY_3.sender(),
        4 => UDP_EI_NOTIFY_4.sender(),
        _ => return,
    };
    let _ = notify.try_send(());
}

pub async fn receive_udp_ei_response(node_index: usize) -> (ClientId, ClientResponse) {
    let receiver = match node_index {
        0 => UDP_EI_RESPONSE_0.receiver(),
        1 => UDP_EI_RESPONSE_1.receiver(),
        2 => UDP_EI_RESPONSE_2.receiver(),
        3 => UDP_EI_RESPONSE_3.receiver(),
        4 => UDP_EI_RESPONSE_4.receiver(),
        _ => panic!("invalid node_index"),
    };
    let bytes = receiver.receive().await;
    deserialize_ei_response(&bytes).unwrap_or((0, ClientResponse::Height(0)))
}

pub struct UdpExternalInterface {
    node_index: usize,
}

impl UdpExternalInterface {
    pub fn new(node_index: usize) -> Self {
        Self { node_index }
    }
}

impl ExternalInterfaceIncoming for UdpExternalInterface {
    type Request = ClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        let bytes = match self.node_index {
            0 => UDP_EI_REQUEST_0.try_receive().ok(),
            1 => UDP_EI_REQUEST_1.try_receive().ok(),
            2 => UDP_EI_REQUEST_2.try_receive().ok(),
            3 => UDP_EI_REQUEST_3.try_receive().ok(),
            4 => UDP_EI_REQUEST_4.try_receive().ok(),
            _ => None,
        }?;
        deserialize_ei_request(&bytes)
    }
}

impl ExternalInterfaceOutgoing for UdpExternalInterface {
    type Response = ClientResponse;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        let bytes = serialize_ei_response(client_id, response);
        let sender = match self.node_index {
            0 => UDP_EI_RESPONSE_0.sender(),
            1 => UDP_EI_RESPONSE_1.sender(),
            2 => UDP_EI_RESPONSE_2.sender(),
            3 => UDP_EI_RESPONSE_3.sender(),
            4 => UDP_EI_RESPONSE_4.sender(),
            _ => return,
        };
        let _ = sender.try_send(bytes);
    }
}
