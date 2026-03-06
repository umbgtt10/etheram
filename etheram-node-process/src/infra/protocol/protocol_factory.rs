// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram_node::builders::protocol_builder::ProtocolBuilder;

pub fn build_protocol() -> Result<BoxedProtocol<()>, String> {
    ProtocolBuilder::default()
        .build()
        .map_err(|error| format!("failed to build protocol: {error:?}"))
}
