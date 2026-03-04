// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::execution::block_commitments::compute_block_commitments;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_etheram_validation::ibft_cluster::IbftCluster;
use etheram_etheram_variants::implementations::ibft::{
    ibft_message::IbftMessage, signature_scheme::SignatureBytes,
};
use std::collections::BTreeMap;

fn sequence(height: u64, round: u64, phase: u64) -> u64 {
    (height * 100) + (round * 10) + phase
}

pub fn block(height: u64, proposer: u64) -> Block {
    Block::new(height, proposer, vec![], [0u8; 32])
}

pub fn block_hash(block: &Block) -> [u8; 32] {
    block.compute_hash()
}

pub fn pre_prepare(height: u64, round: u64, block: &Block) -> IbftMessage {
    IbftMessage::PrePrepare {
        sequence: sequence(height, round, 1),
        height,
        round,
        block: block.clone(),
    }
}

pub fn prepare(height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Prepare {
        sequence: sequence(height, round, 2),
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

pub fn commit(height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Commit {
        sequence: sequence(height, round, 3),
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

pub fn validators() -> Vec<u64> {
    vec![0, 1, 2, 3]
}

pub fn finalize_round_with_block(
    cluster: &mut IbftCluster,
    proposer: u64,
    height: u64,
    round: u64,
    block: &Block,
) {
    let node_count = validators().len();
    let proposed_block_hash = block_hash(block);

    for receiver in 0..node_count {
        cluster.inject_message(receiver, proposer, pre_prepare(height, round, block));
    }
    for receiver in 0..node_count {
        if receiver as u64 != proposer {
            cluster.inject_message(
                receiver,
                proposer,
                prepare(height, round, proposed_block_hash),
            );
        }
    }
    for replica in 0..node_count {
        cluster.drain(replica);
    }
    for sender in 0..node_count {
        if sender as u64 == proposer {
            continue;
        }
        for receiver in 0..node_count {
            if receiver != sender {
                cluster.inject_message(
                    receiver,
                    sender as u64,
                    prepare(height, round, proposed_block_hash),
                );
            }
        }
    }
    cluster.drain_all();
    for sender in 0..node_count {
        for receiver in 0..node_count {
            if receiver != sender {
                cluster.inject_message(
                    receiver,
                    sender as u64,
                    commit(height, round, proposed_block_hash),
                );
            }
        }
    }
    cluster.drain_all();
}

pub fn finalize_round_after_proposer_timer(
    cluster: &mut IbftCluster,
    proposer: usize,
    height: u64,
    round: u64,
    block: &Block,
) {
    let node_count = validators().len();
    let proposed_block_hash = block_hash(block);

    cluster.fire_timer(proposer, TimerEvent::ProposeBlock);
    cluster.drain(proposer);
    for receiver in 0..node_count {
        if receiver != proposer {
            cluster.inject_message(receiver, proposer as u64, pre_prepare(height, round, block));
            cluster.inject_message(
                receiver,
                proposer as u64,
                prepare(height, round, proposed_block_hash),
            );
        }
    }
    for receiver in 0..node_count {
        if receiver != proposer {
            cluster.drain(receiver);
        }
    }
    for sender in 0..node_count {
        if sender != proposer {
            for receiver in 0..node_count {
                if receiver != sender {
                    cluster.inject_message(
                        receiver,
                        sender as u64,
                        prepare(height, round, proposed_block_hash),
                    );
                }
            }
        }
    }
    cluster.drain_all();
    for sender in 0..node_count {
        for receiver in 0..node_count {
            if receiver != sender {
                cluster.inject_message(
                    receiver,
                    sender as u64,
                    commit(height, round, proposed_block_hash),
                );
            }
        }
    }
    cluster.drain_all();
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
