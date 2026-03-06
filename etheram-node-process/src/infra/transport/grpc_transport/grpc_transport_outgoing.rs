// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;

pub struct GrpcTransportOutgoing {
    node_id: PeerId,
    listen_addr: String,
}

impl GrpcTransportOutgoing {
    pub fn new(node_id: PeerId, listen_addr: String) -> Self {
        Self {
            node_id,
            listen_addr,
        }
    }
}

impl TransportOutgoing for GrpcTransportOutgoing {
    type Message = ();

    fn send(&self, _peer_id: PeerId, _message: Self::Message) {
        let _ = self.node_id;
        let _ = &self.listen_addr;
    }
}
