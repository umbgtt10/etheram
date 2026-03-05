// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;

pub struct InMemoryTransportState<M> {
    inboxes: BTreeMap<u64, Vec<(PeerId, M)>>,
}

impl<M> InMemoryTransportState<M> {
    pub fn new() -> Self {
        Self {
            inboxes: BTreeMap::new(),
        }
    }
    pub fn push_message(&mut self, receiver_node_id: u64, from_peer_id: PeerId, message: M) {
        self.inboxes
            .entry(receiver_node_id)
            .or_default()
            .push((from_peer_id, message));
    }
}

impl<M> Default for InMemoryTransportState<M> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InMemoryTransport<M, S>
where
    S: SharedState<InMemoryTransportState<M>>,
{
    node_id: u64,
    state: S,
    marker: PhantomData<M>,
}

impl<M, S> InMemoryTransport<M, S>
where
    S: SharedState<InMemoryTransportState<M>>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|state| {
            state.inboxes.entry(node_id).or_default();
        });
        Self {
            node_id,
            state,
            marker: PhantomData,
        }
    }
}

impl<M, S> TransportIncoming for InMemoryTransport<M, S>
where
    M: Clone + Send + 'static,
    S: SharedState<InMemoryTransportState<M>>,
{
    type Message = M;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        self.state.with_mut(|state| {
            if let Some(inbox) = state.inboxes.get_mut(&self.node_id) {
                if !inbox.is_empty() {
                    return Some(inbox.remove(0));
                }
            }
            None
        })
    }
}

impl<M, S> TransportOutgoing for InMemoryTransport<M, S>
where
    M: Clone + Send + 'static,
    S: SharedState<InMemoryTransportState<M>>,
{
    type Message = M;

    fn send(&self, peer_id: PeerId, message: Self::Message) {
        self.state.with_mut(|state| {
            state
                .inboxes
                .entry(peer_id)
                .or_default()
                .push((self.node_id, message));
        });
    }
}
