// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram_node::builders::external_interface_incoming_builder::ExternalInterfaceIncomingBuilder;
use etheram_node::incoming::external_interface::client_request::ClientRequest;

pub fn build_external_interface_incoming(
) -> Result<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>, String> {
    ExternalInterfaceIncomingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build external interface incoming: {error:?}"))
}
