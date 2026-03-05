// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Balance;
use etheram_node::common_types::types::Gas;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::common_types::types::Nonce;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireTransaction {
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub gas_limit: Gas,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

impl From<Transaction> for WireTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_limit: tx.gas_limit,
            nonce: tx.nonce,
            data: tx.data,
        }
    }
}

impl From<WireTransaction> for Transaction {
    fn from(wire: WireTransaction) -> Self {
        Self {
            from: wire.from,
            to: wire.to,
            value: wire.value,
            gas_limit: wire.gas_limit,
            nonce: wire.nonce,
            data: wire.data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireBlock {
    pub height: Height,
    pub proposer: u64,
    pub transactions: Vec<WireTransaction>,
    pub state_root: Hash,
    pub post_state_root: Hash,
    pub receipts_root: Hash,
}

impl From<Block> for WireBlock {
    fn from(block: Block) -> Self {
        Self {
            height: block.height,
            proposer: block.proposer,
            transactions: block
                .transactions
                .into_iter()
                .map(WireTransaction::from)
                .collect(),
            state_root: block.state_root,
            post_state_root: block.post_state_root,
            receipts_root: block.receipts_root,
        }
    }
}

impl From<WireBlock> for Block {
    fn from(wire: WireBlock) -> Self {
        Self {
            height: wire.height,
            proposer: wire.proposer,
            transactions: wire
                .transactions
                .into_iter()
                .map(Transaction::from)
                .collect(),
            state_root: wire.state_root,
            post_state_root: wire.post_state_root,
            receipts_root: wire.receipts_root,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WirePreparedCertificate {
    pub height: Height,
    pub round: u64,
    pub block_hash: Hash,
    pub signed_prepares: Vec<(u64, Vec<u8>)>,
}

impl From<PreparedCertificate> for WirePreparedCertificate {
    fn from(cert: PreparedCertificate) -> Self {
        Self {
            height: cert.height,
            round: cert.round,
            block_hash: cert.block_hash,
            signed_prepares: cert
                .signed_prepares
                .into_iter()
                .map(|(p, sig)| (p, sig.as_bytes().to_vec()))
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
                .map(|(p, sig)| (p, SignatureBytes::from_slice(&sig)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireIbftMessage {
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
    fn from(msg: IbftMessage) -> Self {
        match msg {
            IbftMessage::PrePrepare {
                sequence,
                height,
                round,
                block,
            } => WireIbftMessage::PrePrepare {
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
            } => WireIbftMessage::Prepare {
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
            } => WireIbftMessage::Commit {
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
            } => WireIbftMessage::ViewChange {
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
            } => WireIbftMessage::NewView {
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
            } => IbftMessage::PrePrepare {
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
            } => IbftMessage::Prepare {
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
            } => IbftMessage::Commit {
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
            } => IbftMessage::ViewChange {
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
            } => IbftMessage::NewView {
                sequence,
                height,
                round,
                prepared_certificate: prepared_certificate.map(PreparedCertificate::from),
                view_change_senders,
            },
        }
    }
}

pub fn serialize(msg: &IbftMessage) -> Result<alloc::vec::Vec<u8>, postcard::Error> {
    let wire = WireIbftMessage::from(msg.clone());
    postcard::to_allocvec(&wire)
}

pub fn deserialize(bytes: &[u8]) -> Result<IbftMessage, postcard::Error> {
    let wire: WireIbftMessage = postcard::from_bytes(bytes)?;
    Ok(IbftMessage::from(wire))
}
