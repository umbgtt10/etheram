// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::marker::PhantomData;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message::RaftMessage;

pub struct NoOpRaftTransport<P> {
    _phantom: PhantomData<P>,
}

impl<P: Clone + 'static> NoOpRaftTransport<P> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<P: Clone + 'static> Default for NoOpRaftTransport<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Clone + 'static> TransportIncoming for NoOpRaftTransport<P> {
    type Message = RaftMessage<P>;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        None
    }
}

impl<P: Clone + 'static> TransportOutgoing for NoOpRaftTransport<P> {
    type Message = RaftMessage<P>;

    fn send(&self, _to: PeerId, _message: Self::Message) {}
}
