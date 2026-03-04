// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use core::marker::PhantomData;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use etheram_etheram_variants::implementations::shared_state::SharedState;

pub struct OutboxTransport<M, S>
where
    S: SharedState<Vec<(PeerId, M)>>,
{
    state: S,
    _marker: PhantomData<M>,
}

impl<M, S> OutboxTransport<M, S>
where
    S: SharedState<Vec<(PeerId, M)>>,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            _marker: PhantomData,
        }
    }
}

impl<M: 'static, S: 'static> TransportOutgoing for OutboxTransport<M, S>
where
    S: SharedState<Vec<(PeerId, M)>>,
{
    type Message = M;

    fn send(&self, peer_id: PeerId, message: Self::Message) {
        self.state.with_mut(|buf| buf.push((peer_id, message)));
    }
}
