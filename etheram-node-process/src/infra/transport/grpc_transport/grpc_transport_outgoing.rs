// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_bus::enqueue_to;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;

pub struct GrpcTransportOutgoing {
    node_id: PeerId,
}

impl GrpcTransportOutgoing {
    pub fn new(node_id: PeerId, _listen_addr: String) -> Self {
        Self { node_id }
    }
}

impl TransportOutgoing for GrpcTransportOutgoing {
    type Message = ();

    fn send(&self, peer_id: PeerId, _message: Self::Message) {
        enqueue_to(peer_id, self.node_id);
    }
}
