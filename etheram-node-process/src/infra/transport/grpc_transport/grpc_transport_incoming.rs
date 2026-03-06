// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_bus::dequeue_for;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;

pub struct GrpcTransportIncoming {
    node_id: PeerId,
}

impl GrpcTransportIncoming {
    pub fn new(node_id: PeerId, _listen_addr: String) -> Self {
        Self { node_id }
    }
}

impl TransportIncoming for GrpcTransportIncoming {
    type Message = ();

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        dequeue_for(self.node_id)
    }
}
