// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram_node::builders::execution_engine_builder::ExecutionEngineBuilder;
use etheram_node::builders::protocol_builder::ProtocolBuilder;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::variants::ProtocolVariant;

pub fn build_protocol(validators: &[u64]) -> Result<BoxedProtocol<IbftMessage>, String> {
    let execution_engine = ExecutionEngineBuilder::default()
        .build()
        .map_err(|error| format!("failed to build execution engine: {error:?}"))?;

    ProtocolBuilder::<IbftMessage>::new()
        .with_execution_engine(execution_engine)
        .with_variant(ProtocolVariant::Ibft {
            validators: validators.to_vec(),
        })
        .build()
        .map_err(|error| format!("failed to build protocol: {error:?}"))
}
