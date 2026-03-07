// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::codec::block_codec::WireBlock;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use serde::Deserialize;
use serde::Serialize;
use std::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WirePreparedCertificate {
    height: Height,
    round: u64,
    block_hash: Hash,
    signed_prepares: Vec<(u64, Vec<u8>)>,
}

impl From<PreparedCertificate> for WirePreparedCertificate {
    fn from(certificate: PreparedCertificate) -> Self {
        Self {
            height: certificate.height,
            round: certificate.round,
            block_hash: certificate.block_hash,
            signed_prepares: certificate
                .signed_prepares
                .into_iter()
                .map(|(peer_id, signature)| (peer_id, signature.as_bytes().to_vec()))
                .collect(),
        }
    }
}

impl From<WirePreparedCertificate> for PreparedCertificate {
    fn from(wire: WirePreparedCertificate) -> Self {
        Self {
            height: wire.height,
            round: wire.round,
            block_hash: wire.block_hash,
            signed_prepares: wire
                .signed_prepares
                .into_iter()
                .map(|(peer_id, signature)| (peer_id, SignatureBytes::from_slice(&signature)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireIbftMessage {
    PrePrepare {
        sequence: u64,
        height: Height,
        round: u64,
        block: WireBlock,
    },
    Prepare {
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: Vec<u8>,
    },
    Commit {
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: Vec<u8>,
    },
    ViewChange {
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: Option<WirePreparedCertificate>,
    },
    NewView {
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: Option<WirePreparedCertificate>,
        view_change_senders: Vec<u64>,
    },
}

impl From<IbftMessage> for WireIbftMessage {
    fn from(message: IbftMessage) -> Self {
        match message {
            IbftMessage::PrePrepare {
                sequence,
                height,
                round,
                block,
            } => Self::PrePrepare {
                sequence,
                height,
                round,
                block: WireBlock::from(block),
            },
            IbftMessage::Prepare {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            } => Self::Prepare {
                sequence,
                height,
                round,
                block_hash,
                sender_signature: sender_signature.as_bytes().to_vec(),
            },
            IbftMessage::Commit {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            } => Self::Commit {
                sequence,
                height,
                round,
                block_hash,
                sender_signature: sender_signature.as_bytes().to_vec(),
            },
            IbftMessage::ViewChange {
                sequence,
                height,
                round,
                prepared_certificate,
            } => Self::ViewChange {
                sequence,
                height,
                round,
                prepared_certificate: prepared_certificate.map(WirePreparedCertificate::from),
            },
            IbftMessage::NewView {
                sequence,
                height,
                round,
                prepared_certificate,
                view_change_senders,
            } => Self::NewView {
                sequence,
                height,
                round,
                prepared_certificate: prepared_certificate.map(WirePreparedCertificate::from),
                view_change_senders,
            },
        }
    }
}

impl From<WireIbftMessage> for IbftMessage {
    fn from(wire: WireIbftMessage) -> Self {
        match wire {
            WireIbftMessage::PrePrepare {
                sequence,
                height,
                round,
                block,
            } => Self::PrePrepare {
                sequence,
                height,
                round,
                block: Block::from(block),
            },
            WireIbftMessage::Prepare {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            } => Self::Prepare {
                sequence,
                height,
                round,
                block_hash,
                sender_signature: SignatureBytes::from_slice(&sender_signature),
            },
            WireIbftMessage::Commit {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            } => Self::Commit {
                sequence,
                height,
                round,
                block_hash,
                sender_signature: SignatureBytes::from_slice(&sender_signature),
            },
            WireIbftMessage::ViewChange {
                sequence,
                height,
                round,
                prepared_certificate,
            } => Self::ViewChange {
                sequence,
                height,
                round,
                prepared_certificate: prepared_certificate.map(PreparedCertificate::from),
            },
            WireIbftMessage::NewView {
                sequence,
                height,
                round,
                prepared_certificate,
                view_change_senders,
            } => Self::NewView {
                sequence,
                height,
                round,
                prepared_certificate: prepared_certificate.map(PreparedCertificate::from),
                view_change_senders,
            },
        }
    }
}

pub fn serialize(message: &IbftMessage) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(&WireIbftMessage::from(message.clone()))
}

pub fn deserialize(bytes: &[u8]) -> Result<IbftMessage, postcard::Error> {
    let wire: WireIbftMessage = postcard::from_bytes(bytes)?;
    Ok(IbftMessage::from(wire))
}
