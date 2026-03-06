// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use etheram_node::builders::external_interface_outgoing_builder::ExternalInterfaceOutgoingBuilder;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;

pub fn build_external_interface_outgoing(
) -> Result<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>, String> {
    ExternalInterfaceOutgoingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build external interface outgoing: {error:?}"))
}
