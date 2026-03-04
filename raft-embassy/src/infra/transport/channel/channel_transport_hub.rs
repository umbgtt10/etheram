// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message::RaftMessage;

const TRANSPORT_CAPACITY: usize = 64;

type TransportPayload = Vec<u8>;
type TransportMessage = (PeerId, RaftMessage<TransportPayload>);
type TransportChannel = Channel<CriticalSectionRawMutex, TransportMessage, TRANSPORT_CAPACITY>;

pub struct ChannelTransportHub {
    channels: [TransportChannel; MAX_NODES],
}

impl ChannelTransportHub {
    pub const fn new() -> Self {
        Self {
            channels: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
        }
    }

    pub async fn send(
        &self,
        to_node: usize,
        from_peer: PeerId,
        message: RaftMessage<TransportPayload>,
    ) {
        self.channels[to_node].send((from_peer, message)).await;
    }

    pub async fn receive(&self, node_index: usize) -> TransportMessage {
        self.channels[node_index].receive().await
    }
}

pub static TRANSPORT_HUB: ChannelTransportHub = ChannelTransportHub::new();
