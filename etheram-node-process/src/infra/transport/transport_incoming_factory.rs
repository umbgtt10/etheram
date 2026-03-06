// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::transport_incoming_adapter::TransportIncomingAdapter;
use etheram_node::builders::transport_incoming_builder::TransportIncomingBuilder;

pub fn build_transport_incoming() -> Result<Box<dyn TransportIncomingAdapter<()>>, String> {
    TransportIncomingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build transport incoming: {error:?}"))
}
