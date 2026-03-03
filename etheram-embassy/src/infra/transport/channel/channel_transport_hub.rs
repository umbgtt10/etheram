// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use barechain_core::types::PeerId;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

const TRANSPORT_CAPACITY: usize = 64;

type TransportMessage = (PeerId, IbftMessage);
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

    pub async fn send(&self, to_node: usize, from_peer: PeerId, message: IbftMessage) {
        self.channels[to_node].send((from_peer, message)).await;
    }

    pub async fn receive(&self, node_index: usize) -> TransportMessage {
        self.channels[node_index].receive().await
    }
}

pub static TRANSPORT_HUB: ChannelTransportHub = ChannelTransportHub::new();
