// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;

#[derive(Clone, Copy)]
pub struct NoOpTransport;

impl TransportIncoming for NoOpTransport {
    type Message = ();
    fn poll(&self) -> Option<(u64, Self::Message)> {
        None
    }
}

impl TransportOutgoing for NoOpTransport {
    type Message = ();
    fn send(&self, _peer_id: u64, _message: Self::Message) {}
}
