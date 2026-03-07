// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_bus::GrpcExternalInterfaceBus;
use crate::infra::external_interface::grpc_external_interface_outgoing::GrpcExternalInterfaceOutgoing;
use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;

pub fn build_external_interface_outgoing(
    bus: GrpcExternalInterfaceBus,
) -> Result<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>, String> {
    Ok(Box::new(GrpcExternalInterfaceOutgoing::new(bus)))
}
