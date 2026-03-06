// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;
use etheram_node::builders::transport_outgoing_builder::TransportOutgoingBuilder;

pub fn build_transport_outgoing() -> Result<Box<dyn TransportOutgoingAdapter<()>>, String> {
    TransportOutgoingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build transport outgoing: {error:?}"))
}
