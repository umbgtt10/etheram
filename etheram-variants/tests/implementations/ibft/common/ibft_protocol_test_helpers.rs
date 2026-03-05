// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::types::PeerId;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Hash;
use etheram_node::context::context_dto::Context;
use etheram_node::execution::block_commitments::compute_block_commitments;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_variants::implementations::ibft::signature_scheme::SignatureScheme;
use etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use std::collections::BTreeMap;

pub struct AlternateSignatureScheme;

impl SignatureScheme for AlternateSignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        SignatureBytes::zeroed()
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        true
    }
}

pub fn setup_protocol() -> IbftProtocol {
    IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)))
}

pub fn setup_protocol_with_validator_updates(updates: Vec<ValidatorSetUpdate>) -> IbftProtocol {
    IbftProtocol::new_with_validator_updates(
        vec![0, 1, 2, 3],
        Box::new(MockSignatureScheme::new(0)),
        updates,
    )
}

pub fn setup_context(peer_id: u64, height: u64) -> Context {
    Context::new(peer_id, height, [0u8; 32])
}

pub fn setup_restored_protocol(wal: ConsensusWal) -> IbftProtocol {
    IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal)
}

pub fn setup_wal_base() -> ConsensusWal {
    ConsensusWal {
        height: 0,
        round: 0,
        active_validators: vec![0, 1, 2, 3],
        scheduled_validator_updates: BTreeMap::new(),
        pending_block: None,
        observed_pre_prepares: BTreeMap::new(),
        prepared_certificate: None,
        prepare_votes: BTreeMap::new(),
        commit_votes: BTreeMap::new(),
        rejected_block_hashes: vec![],
        malicious_senders: vec![],
        view_change_votes: BTreeMap::new(),
        seen_messages: vec![],
        highest_seen_sequence: BTreeMap::new(),
        prepare_sent: false,
        commit_sent: false,
        new_view_sent_round: None,
        next_outgoing_sequence: 0,
        prepare_signatures: vec![],
    }
}

pub fn setup_wal_with<F>(mutator: F) -> ConsensusWal
where
    F: FnOnce(&mut ConsensusWal),
{
    let mut wal = setup_wal_base();
    mutator(&mut wal);
    wal
}

pub fn setup_after_propose() -> (IbftProtocol, Context, Hash) {
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let block_hash = match actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected Prepare action"),
    };
    (protocol, ctx, block_hash)
}

pub fn setup_after_propose_with_tx() -> (IbftProtocol, Context, Hash, Transaction) {
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = Context::new(0, 0, [0u8; 32]);
    ctx.accounts.insert(
        from,
        Account {
            balance: 1000,
            nonce: 0,
        },
    );
    ctx.accounts.insert(
        to,
        Account {
            balance: 0,
            nonce: 0,
        },
    );
    ctx.pending_txs.push(tx.clone());
    let mut protocol = setup_protocol();
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let block_hash = match actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected Prepare action"),
    };
    (protocol, ctx, block_hash, tx)
}

pub fn build_block_with_commitments(
    height: u64,
    proposer: u64,
    transactions: Vec<Transaction>,
    state_root: Hash,
    accounts: &BTreeMap<Address, Account>,
    contract_storage: &BTreeMap<(Address, Hash), Hash>,
    engine: &dyn ExecutionEngine,
) -> Block {
    let mut block = Block::new(height, proposer, transactions, state_root);
    let (post_state_root, receipts_root) =
        compute_block_commitments(&block, accounts, contract_storage, engine);
    block.post_state_root = post_state_root;
    block.receipts_root = receipts_root;
    block
}
