// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;

pub struct GrpcTransportIncoming {
    node_id: PeerId,
    listen_addr: String,
}

impl GrpcTransportIncoming {
    pub fn new(node_id: PeerId, listen_addr: String) -> Self {
        Self {
            node_id,
            listen_addr,
        }
    }
}

impl TransportIncoming for GrpcTransportIncoming {
    type Message = ();

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        let _ = self.node_id;
        let _ = &self.listen_addr;
        None
    }
}
