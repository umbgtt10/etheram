// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::udp::wire_raft_message::deserialize;
use crate::infra::transport::udp::wire_raft_message::serialize;
use alloc::vec::Vec;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::IpEndpoint;
use embassy_net::Ipv4Address;
use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use embassy_sync::channel::Sender;
use embassy_time::Duration;
use embassy_time::Timer;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message::RaftMessage;
use serde::Deserialize;
use serde::Serialize;

const CHANNEL_CAPACITY: usize = 64;
const BASE_PORT: u16 = 9100;
const MAX_PACKET_SIZE: usize = 2048;

type P = alloc::vec::Vec<u8>;
type RaftChannelMessage = (PeerId, RaftMessage<P>);
type RaftTransportChannel = Channel<CriticalSectionRawMutex, RaftChannelMessage, CHANNEL_CAPACITY>;

pub type RaftMessageSender =
    Sender<'static, CriticalSectionRawMutex, RaftChannelMessage, CHANNEL_CAPACITY>;

pub type RaftMessageReceiver =
    Receiver<'static, CriticalSectionRawMutex, RaftChannelMessage, CHANNEL_CAPACITY>;

static INBOX_0: RaftTransportChannel = Channel::new();
static INBOX_1: RaftTransportChannel = Channel::new();
static INBOX_2: RaftTransportChannel = Channel::new();
static INBOX_3: RaftTransportChannel = Channel::new();
static INBOX_4: RaftTransportChannel = Channel::new();

static OUTBOX_0: RaftTransportChannel = Channel::new();
static OUTBOX_1: RaftTransportChannel = Channel::new();
static OUTBOX_2: RaftTransportChannel = Channel::new();
static OUTBOX_3: RaftTransportChannel = Channel::new();
static OUTBOX_4: RaftTransportChannel = Channel::new();

pub fn inbox_endpoints(node_index: usize) -> (RaftMessageSender, RaftMessageReceiver) {
    match node_index {
        0 => (INBOX_0.sender(), INBOX_0.receiver()),
        1 => (INBOX_1.sender(), INBOX_1.receiver()),
        2 => (INBOX_2.sender(), INBOX_2.receiver()),
        3 => (INBOX_3.sender(), INBOX_3.receiver()),
        4 => (INBOX_4.sender(), INBOX_4.receiver()),
        _ => panic!("invalid node_index"),
    }
}

pub fn outbox_endpoints(node_index: usize) -> (RaftMessageSender, RaftMessageReceiver) {
    match node_index {
        0 => (OUTBOX_0.sender(), OUTBOX_0.receiver()),
        1 => (OUTBOX_1.sender(), OUTBOX_1.receiver()),
        2 => (OUTBOX_2.sender(), OUTBOX_2.receiver()),
        3 => (OUTBOX_3.sender(), OUTBOX_3.receiver()),
        4 => (OUTBOX_4.sender(), OUTBOX_4.receiver()),
        _ => panic!("invalid node_index"),
    }
}

pub struct UdpInboundRaftTransport {
    receiver: RaftMessageReceiver,
}

impl UdpInboundRaftTransport {
    pub fn new(receiver: RaftMessageReceiver) -> Self {
        Self { receiver }
    }

    pub fn into_receiver(self) -> RaftMessageReceiver {
        self.receiver
    }
}

impl TransportIncoming for UdpInboundRaftTransport {
    type Message = RaftMessage<P>;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        self.receiver.try_receive().ok()
    }
}

pub struct UdpOutboundRaftTransport {
    sender: RaftMessageSender,
}

impl UdpOutboundRaftTransport {
    pub fn new(sender: RaftMessageSender) -> Self {
        Self { sender }
    }
}

impl TransportOutgoing for UdpOutboundRaftTransport {
    type Message = RaftMessage<P>;

    fn send(&self, peer_id: PeerId, message: Self::Message) {
        if self.sender.try_send((peer_id, message)).is_err() {
            crate::info!("udp transport outbox full, dropping message to {}", peer_id);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    from: PeerId,
    message_bytes: Vec<u8>,
}

fn peer_endpoint(peer_id: PeerId) -> IpEndpoint {
    IpEndpoint::new(
        Ipv4Address::new(10, 0, 1, (peer_id + 1) as u8).into(),
        BASE_PORT + peer_id as u16,
    )
}

pub async fn run_udp_listener(node_index: usize, stack: Stack<'static>, sender: RaftMessageSender) {
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; MAX_PACKET_SIZE * 2];
    let mut tx_buffer = [0u8; MAX_PACKET_SIZE * 2];
    let mut recv_buf = [0u8; MAX_PACKET_SIZE];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let port = BASE_PORT + node_index as u16;
    if socket.bind(port).is_err() {
        return;
    }

    loop {
        match socket.recv_from(&mut recv_buf).await {
            Ok((len, _from_addr)) => {
                let envelope: Envelope = match postcard::from_bytes(&recv_buf[..len]) {
                    Ok(env) => env,
                    Err(_) => continue,
                };
                let message = match deserialize(&envelope.message_bytes) {
                    Ok(msg) => msg,
                    Err(_) => continue,
                };
                if sender.try_send((envelope.from, message)).is_err() {
                    crate::info!(
                        "udp transport inbox full for node {}, dropping inbound message",
                        node_index
                    );
                }
            }
            Err(_) => {
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}

pub async fn run_udp_sender(
    node_index: usize,
    stack: Stack<'static>,
    receiver: RaftMessageReceiver,
) {
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; MAX_PACKET_SIZE * 2];
    let mut tx_buffer = [0u8; MAX_PACKET_SIZE * 2];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    if socket.bind(0).is_err() {
        return;
    }

    loop {
        let (peer_id, message) = receiver.receive().await;

        let message_bytes = match serialize(&message) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        let envelope = Envelope {
            from: node_index as PeerId,
            message_bytes,
        };

        let bytes = match postcard::to_allocvec(&envelope) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let target = peer_endpoint(peer_id);
        if socket.send_to(&bytes, target).await.is_err() {
            crate::info!(
                "udp transport send_to failed from {} to {}",
                node_index,
                peer_id
            );
        }
    }
}
