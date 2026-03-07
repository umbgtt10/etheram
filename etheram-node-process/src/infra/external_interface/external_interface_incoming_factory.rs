// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_bus::GrpcExternalInterfaceBus;
use crate::infra::external_interface::grpc_external_interface_incoming::GrpcExternalInterfaceIncoming;
use etheram_core::node_common::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

pub fn build_external_interface_incoming(
    bus: GrpcExternalInterfaceBus,
) -> Result<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>, String> {
    Ok(Box::new(GrpcExternalInterfaceIncoming::new(bus)))
}
