// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::string::String;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::types::ClientId;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use serde::Deserialize;
use serde::Serialize;

const EI_CAPACITY: usize = 16;

type EiPacketChannel = Channel<CriticalSectionRawMutex, Vec<u8>, EI_CAPACITY>;
type EiNotifyChannel = Channel<CriticalSectionRawMutex, (), 1>;

pub type UdpRaftEiNotifyReceiver = Receiver<'static, CriticalSectionRawMutex, (), 1>;

static UDP_RAFT_EI_NOTIFY_0: EiNotifyChannel = Channel::new();
static UDP_RAFT_EI_NOTIFY_1: EiNotifyChannel = Channel::new();
static UDP_RAFT_EI_NOTIFY_2: EiNotifyChannel = Channel::new();
static UDP_RAFT_EI_NOTIFY_3: EiNotifyChannel = Channel::new();
static UDP_RAFT_EI_NOTIFY_4: EiNotifyChannel = Channel::new();

static UDP_RAFT_EI_REQUEST_0: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_REQUEST_1: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_REQUEST_2: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_REQUEST_3: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_REQUEST_4: EiPacketChannel = Channel::new();

static UDP_RAFT_EI_RESPONSE_0: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_RESPONSE_1: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_RESPONSE_2: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_RESPONSE_3: EiPacketChannel = Channel::new();
static UDP_RAFT_EI_RESPONSE_4: EiPacketChannel = Channel::new();

#[derive(Serialize, Deserialize)]
enum WireRaftClientRequest {
    Command(Vec<u8>),
    Query(String),
}

#[derive(Serialize, Deserialize)]
enum WireRaftClientResponse {
    Applied(Vec<u8>),
    QueryResult(Vec<u8>),
    NotLeader(Option<u64>),
    Timeout,
}

#[derive(Serialize, Deserialize)]
struct WireEiRequestPacket {
    client_id: ClientId,
    payload: WireRaftClientRequest,
}

#[derive(Serialize, Deserialize)]
struct WireEiResponsePacket {
    client_id: ClientId,
    payload: WireRaftClientResponse,
}

fn serialize_ei_request(client_id: ClientId, request: RaftClientRequest) -> Vec<u8> {
    let wire_payload = match request {
        RaftClientRequest::Command(data) => WireRaftClientRequest::Command(data),
        RaftClientRequest::Query(q) => WireRaftClientRequest::Query(q),
    };
    let packet = WireEiRequestPacket {
        client_id,
        payload: wire_payload,
    };
    postcard::to_allocvec(&packet).unwrap_or_default()
}

fn deserialize_ei_request(bytes: &[u8]) -> Option<(ClientId, RaftClientRequest)> {
    let packet: WireEiRequestPacket = postcard::from_bytes(bytes).ok()?;
    let request = match packet.payload {
        WireRaftClientRequest::Command(data) => RaftClientRequest::Command(data),
        WireRaftClientRequest::Query(q) => RaftClientRequest::Query(q),
    };
    Some((packet.client_id, request))
}

fn serialize_ei_response(client_id: ClientId, response: RaftClientResponse) -> Vec<u8> {
    let wire_payload = match response {
        RaftClientResponse::Applied(data) => WireRaftClientResponse::Applied(data),
        RaftClientResponse::QueryResult(data) => WireRaftClientResponse::QueryResult(data),
        RaftClientResponse::NotLeader(peer) => WireRaftClientResponse::NotLeader(peer),
        RaftClientResponse::Timeout => WireRaftClientResponse::Timeout,
    };
    let packet = WireEiResponsePacket {
        client_id,
        payload: wire_payload,
    };
    postcard::to_allocvec(&packet).unwrap_or_default()
}

fn deserialize_ei_response(bytes: &[u8]) -> Option<(ClientId, RaftClientResponse)> {
    let packet: WireEiResponsePacket = postcard::from_bytes(bytes).ok()?;
    let response = match packet.payload {
        WireRaftClientResponse::Applied(data) => RaftClientResponse::Applied(data),
        WireRaftClientResponse::QueryResult(data) => RaftClientResponse::QueryResult(data),
        WireRaftClientResponse::NotLeader(peer) => RaftClientResponse::NotLeader(peer),
        WireRaftClientResponse::Timeout => RaftClientResponse::Timeout,
    };
    Some((packet.client_id, response))
}

pub fn udp_raft_ei_notify_receiver(node_index: usize) -> UdpRaftEiNotifyReceiver {
    match node_index {
        0 => UDP_RAFT_EI_NOTIFY_0.receiver(),
        1 => UDP_RAFT_EI_NOTIFY_1.receiver(),
        2 => UDP_RAFT_EI_NOTIFY_2.receiver(),
        3 => UDP_RAFT_EI_NOTIFY_3.receiver(),
        4 => UDP_RAFT_EI_NOTIFY_4.receiver(),
        _ => panic!("invalid node_index"),
    }
}

pub fn send_udp_raft_ei_request(
    node_index: usize,
    client_id: ClientId,
    request: RaftClientRequest,
) {
    let bytes = serialize_ei_request(client_id, request);
    let sender = match node_index {
        0 => UDP_RAFT_EI_REQUEST_0.sender(),
        1 => UDP_RAFT_EI_REQUEST_1.sender(),
        2 => UDP_RAFT_EI_REQUEST_2.sender(),
        3 => UDP_RAFT_EI_REQUEST_3.sender(),
        4 => UDP_RAFT_EI_REQUEST_4.sender(),
        _ => return,
    };
    if sender.try_send(bytes).is_err() {
        crate::info!(
            "udp external interface request queue full for node {}",
            node_index
        );
    }
    let notify = match node_index {
        0 => UDP_RAFT_EI_NOTIFY_0.sender(),
        1 => UDP_RAFT_EI_NOTIFY_1.sender(),
        2 => UDP_RAFT_EI_NOTIFY_2.sender(),
        3 => UDP_RAFT_EI_NOTIFY_3.sender(),
        4 => UDP_RAFT_EI_NOTIFY_4.sender(),
        _ => return,
    };
    if notify.try_send(()).is_err() {
        crate::info!(
            "udp external interface notify queue full for node {}",
            node_index
        );
    }
}

pub async fn receive_udp_raft_ei_response(node_index: usize) -> (ClientId, RaftClientResponse) {
    let receiver = match node_index {
        0 => UDP_RAFT_EI_RESPONSE_0.receiver(),
        1 => UDP_RAFT_EI_RESPONSE_1.receiver(),
        2 => UDP_RAFT_EI_RESPONSE_2.receiver(),
        3 => UDP_RAFT_EI_RESPONSE_3.receiver(),
        4 => UDP_RAFT_EI_RESPONSE_4.receiver(),
        _ => panic!("invalid node_index"),
    };
    let bytes = receiver.receive().await;
    deserialize_ei_response(&bytes).unwrap_or((0, RaftClientResponse::Timeout))
}

pub struct UdpRaftExternalInterface {
    node_index: usize,
}

impl UdpRaftExternalInterface {
    pub fn new(node_index: usize) -> Self {
        Self { node_index }
    }
}

impl ExternalInterfaceIncoming for UdpRaftExternalInterface {
    type Request = RaftClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        let bytes = match self.node_index {
            0 => UDP_RAFT_EI_REQUEST_0.try_receive().ok(),
            1 => UDP_RAFT_EI_REQUEST_1.try_receive().ok(),
            2 => UDP_RAFT_EI_REQUEST_2.try_receive().ok(),
            3 => UDP_RAFT_EI_REQUEST_3.try_receive().ok(),
            4 => UDP_RAFT_EI_REQUEST_4.try_receive().ok(),
            _ => None,
        }?;
        deserialize_ei_request(&bytes)
    }
}

impl ExternalInterfaceOutgoing for UdpRaftExternalInterface {
    type Response = RaftClientResponse;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        let bytes = serialize_ei_response(client_id, response);
        let sender = match self.node_index {
            0 => UDP_RAFT_EI_RESPONSE_0.sender(),
            1 => UDP_RAFT_EI_RESPONSE_1.sender(),
            2 => UDP_RAFT_EI_RESPONSE_2.sender(),
            3 => UDP_RAFT_EI_RESPONSE_3.sender(),
            4 => UDP_RAFT_EI_RESPONSE_4.sender(),
            _ => return,
        };
        if sender.try_send(bytes).is_err() {
            crate::info!(
                "udp external interface response queue full for node {}",
                self.node_index
            );
        }
    }
}
