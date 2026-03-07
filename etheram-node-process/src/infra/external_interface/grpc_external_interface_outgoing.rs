// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_bus::GrpcExternalInterfaceBus;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;

#[derive(Clone)]
pub struct GrpcExternalInterfaceOutgoing {
    bus: GrpcExternalInterfaceBus,
}

impl GrpcExternalInterfaceOutgoing {
    pub fn new(bus: GrpcExternalInterfaceBus) -> Self {
        Self { bus }
    }
}

impl ExternalInterfaceOutgoing for GrpcExternalInterfaceOutgoing {
    type Response = ClientResponse;

    fn send_response(&self, client_id: u64, response: Self::Response) {
        self.bus.store_response(client_id, response);
    }
}
