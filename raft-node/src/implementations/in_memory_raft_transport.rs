// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;

pub struct InMemoryRaftTransportState<P> {
    inboxes: BTreeMap<u64, Vec<(PeerId, RaftMessage<P>)>>,
}

impl<P> InMemoryRaftTransportState<P> {
    pub fn new() -> Self {
        Self {
            inboxes: BTreeMap::new(),
        }
    }

    pub fn push_message(
        &mut self,
        receiver_node_id: u64,
        from_peer_id: PeerId,
        message: RaftMessage<P>,
    ) {
        self.inboxes
            .entry(receiver_node_id)
            .or_default()
            .push((from_peer_id, message));
    }
}

impl<P> Default for InMemoryRaftTransportState<P> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InMemoryRaftTransport<P, S>
where
    S: SharedState<InMemoryRaftTransportState<P>>,
{
    node_id: u64,
    state: S,
    marker: PhantomData<P>,
}

impl<P, S> InMemoryRaftTransport<P, S>
where
    S: SharedState<InMemoryRaftTransportState<P>>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|s| {
            s.inboxes.entry(node_id).or_default();
        });
        Self {
            node_id,
            state,
            marker: PhantomData,
        }
    }
}

impl<P: Clone + 'static, S> TransportIncoming for InMemoryRaftTransport<P, S>
where
    S: SharedState<InMemoryRaftTransportState<P>>,
{
    type Message = RaftMessage<P>;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        self.state.with_mut(|s| {
            if let Some(inbox) = s.inboxes.get_mut(&self.node_id) {
                if !inbox.is_empty() {
                    return Some(inbox.remove(0));
                }
            }
            None
        })
    }
}

impl<P: Clone + 'static, S> TransportOutgoing for InMemoryRaftTransport<P, S>
where
    S: SharedState<InMemoryRaftTransportState<P>>,
{
    type Message = RaftMessage<P>;

    fn send(&self, to: PeerId, message: Self::Message) {
        self.state.with_mut(|s| {
            s.inboxes
                .entry(to)
                .or_default()
                .push((self.node_id, message));
        });
    }
}
