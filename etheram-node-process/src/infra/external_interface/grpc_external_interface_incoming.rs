// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_bus::GrpcExternalInterfaceBus;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

#[derive(Clone)]
pub struct GrpcExternalInterfaceIncoming {
    bus: GrpcExternalInterfaceBus,
}

impl GrpcExternalInterfaceIncoming {
    pub fn new(bus: GrpcExternalInterfaceBus) -> Self {
        Self { bus }
    }
}

impl ExternalInterfaceIncoming for GrpcExternalInterfaceIncoming {
    type Request = ClientRequest;

    fn poll_request(&self) -> Option<(u64, Self::Request)> {
        self.bus.poll_request()
    }
}
