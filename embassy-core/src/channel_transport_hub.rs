// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use etheram_core::types::PeerId;

pub type ChannelTransportMessage<M> = (PeerId, M);

pub struct GenericChannelTransportHub<M, const N: usize, const CAPACITY: usize> {
    channels: [Channel<CriticalSectionRawMutex, ChannelTransportMessage<M>, CAPACITY>; N],
}

impl<M, const N: usize, const CAPACITY: usize> GenericChannelTransportHub<M, N, CAPACITY> {
    pub const fn new() -> Self {
        Self {
            channels: [const { Channel::new() }; N],
        }
    }

    pub async fn send(&self, to_node: usize, from_peer: PeerId, message: M) {
        self.channels[to_node].send((from_peer, message)).await;
    }

    pub async fn receive(&self, node_index: usize) -> ChannelTransportMessage<M> {
        self.channels[node_index].receive().await
    }
}

impl<M, const N: usize, const CAPACITY: usize> Default
    for GenericChannelTransportHub<M, N, CAPACITY>
{
    fn default() -> Self {
        Self::new()
    }
}
