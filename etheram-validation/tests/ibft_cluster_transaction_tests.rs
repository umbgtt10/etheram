// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block_hash;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::state_root::compute_state_root;
use etheram::common_types::transaction::Transaction;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::timer::timer_event::TimerEvent;
use std::collections::BTreeMap;

#[test]
fn replicas_commit_block_containing_transaction() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let state_root = compute_state_root(&genesis_accounts);
    let proposed_block = Block::new(0, 0, vec![tx], state_root);
    let proposed_block_hash = block_hash(&proposed_block);

    // Act
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    let stored = cluster.node_stored_block(0, 0).unwrap();
    assert_eq!(stored, proposed_block);
}

#[test]
fn committed_transaction_transfers_balance_between_accounts() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let state_root = compute_state_root(&genesis_accounts);
    let proposed_block = Block::new(0, 0, vec![tx], state_root);
    let proposed_block_hash = block_hash(&proposed_block);

    // Act
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(900));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(100));
}

#[test]
fn two_consecutive_heights_each_with_transaction_final_balances_are_correct() {
    // Arrange
    let from: [u8; 20] = [1u8; 20];
    let to: [u8; 20] = [2u8; 20];
    let tx_zero = Transaction::transfer(from, to, 100, 21_000, 0);
    let tx_one = Transaction::transfer(to, from, 50, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1_000)]);
    for node in 0..4usize {
        cluster.submit_request(node, 1, ClientRequest::SubmitTransaction(tx_zero.clone()));
        cluster.drain(node);
    }
    let genesis_accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let state_root_genesis = compute_state_root(&genesis_accounts);
    let block_zero = Block::new(0, 0, vec![tx_zero], state_root_genesis);
    let block_zero_hash = block_hash(&block_zero);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &block_zero));
        cluster.inject_message(receiver, 0, prepare(0, 0, block_zero_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, block_zero_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, block_zero_hash));
            }
        }
    }
    cluster.drain_all();
    assert_eq!(cluster.node_height(0), 1);
    let accounts_after_zero = BTreeMap::from([
        (
            from,
            Account {
                balance: 900,
                nonce: 1,
            },
        ),
        (to, Account::new(100)),
    ]);
    let state_root_one = compute_state_root(&accounts_after_zero);
    for node in 0..4usize {
        cluster.submit_request(node, 2, ClientRequest::SubmitTransaction(tx_one.clone()));
        cluster.drain(node);
    }
    let block_one = Block::new(1, 1, vec![tx_one], state_root_one);
    let block_one_hash = block_hash(&block_one);
    cluster.fire_timer(1, TimerEvent::ProposeBlock);
    cluster.drain(1);
    for receiver in [0usize, 2, 3] {
        cluster.inject_message(receiver, 1, pre_prepare(1, 0, &block_one));
        cluster.inject_message(receiver, 1, prepare(1, 0, block_one_hash));
    }
    for replica in [0usize, 2, 3] {
        cluster.drain(replica);
    }
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(1, 0, block_one_hash));
            }
        }
    }
    cluster.drain_all();

    // Act
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(1, 0, block_one_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(950));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(50));
}

#[test]
fn submit_transaction_after_propose_fires_tx_included_in_next_block() {
    // Arrange
    let from = [10u8; 20];
    let to = [11u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let state_root = compute_state_root(&genesis_accounts);
    let empty_block = Block::new(0, 0, vec![], state_root);
    let empty_block_hash = block_hash(&empty_block);

    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &empty_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, empty_block_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, empty_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, empty_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    let committed_block = cluster.node_stored_block(0, 0).unwrap();
    assert!(committed_block.transactions.is_empty());
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(1000));
}
