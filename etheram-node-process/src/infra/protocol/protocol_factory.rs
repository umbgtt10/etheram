// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use etheram_node::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram_node::builders::execution_engine_builder::ExecutionEngineBuilder;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::ed25519_signature_scheme::Ed25519SignatureScheme;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::wal_writer::WalWriter;

pub fn build_protocol(
    peer_id: PeerId,
    validators: &[u64],
    restored_wal: Option<ConsensusWal>,
    wal_writer: Box<dyn WalWriter>,
) -> Result<BoxedProtocol<IbftMessage>, String> {
    let execution_engine = ExecutionEngineBuilder::default()
        .build()
        .map_err(|error| format!("failed to build execution engine: {error:?}"))?;
    let validators = validators.to_vec();
    let protocol = match restored_wal {
        Some(wal) => IbftProtocol::from_wal(
            validators,
            Box::new(Ed25519SignatureScheme::new(peer_id)),
            wal,
        ),
        None => IbftProtocol::new(validators, Box::new(Ed25519SignatureScheme::new(peer_id))),
    }
    .with_wal_writer(wal_writer)
    .with_execution_engine(execution_engine);
    Ok(Box::new(protocol))
}
