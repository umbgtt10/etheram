// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::action::Action;
use etheram_node::collections::action_collection::ActionCollection;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::implementations::in_memory_cache::InMemoryCache;
use etheram_node::implementations::in_memory_storage::InMemoryStorage;
use etheram_node::state::cache::cache_update::CacheUpdate;
use etheram_node::state::etheram_state::EtheramState;

fn fresh_state() -> EtheramState {
    EtheramState::new(
        Box::new(InMemoryStorage::new()),
        Box::new(InMemoryCache::new()),
    )
}

fn apply<M: Clone>(state: &mut EtheramState, actions: Vec<Action<M>>) {
    state.apply_mutations(&ActionCollection::<Action<M>>::from_vec(actions));
}

#[test]
fn query_height_fresh_state_returns_zero() {
    // Arrange
    let state = fresh_state();

    // Act
    let height = state.query_height();

    // Assert
    assert_eq!(height, 0);
}

#[test]
fn query_height_after_two_increments_returns_two() {
    // Arrange
    let mut state = fresh_state();

    // Act
    apply::<()>(
        &mut state,
        vec![Action::IncrementHeight, Action::IncrementHeight],
    );

    // Assert
    assert_eq!(state.query_height(), 2);
}

#[test]
fn query_account_unknown_address_returns_none() {
    // Arrange
    let state = fresh_state();
    let address: Address = [7u8; 20];

    // Act
    let result = state.query_account(address);

    // Assert
    assert!(result.is_none());
}

#[test]
fn query_account_after_update_mutation_returns_account() {
    // Arrange
    let mut state = fresh_state();
    let address: Address = [3u8; 20];
    let account = Account::new(999);

    // Act
    apply::<()>(
        &mut state,
        vec![Action::UpdateAccount {
            address,
            account: account.clone(),
        }],
    );

    // Assert
    let stored = state.query_account(address).unwrap();
    assert_eq!(stored.balance, 999);
}

#[test]
fn query_block_unknown_height_returns_none() {
    // Arrange
    let state = fresh_state();

    // Act
    let result = state.query_block(42);

    // Assert
    assert!(result.is_none());
}

#[test]
fn query_block_after_store_mutation_returns_block() {
    // Arrange
    let mut state = fresh_state();
    let block = Block::new(0, 0, vec![], [0u8; 32]);

    // Act
    apply::<()>(
        &mut state,
        vec![Action::StoreBlock {
            block: block.clone(),
        }],
    );

    // Assert
    assert_eq!(state.query_block(0), Some(block));
}

#[test]
fn query_pending_fresh_state_returns_empty() {
    // Arrange
    let state = fresh_state();

    // Act
    let txs = state.query_pending();

    // Assert
    assert!(txs.is_empty());
}

#[test]
fn query_pending_after_add_pending_mutation_returns_transaction() {
    // Arrange
    let mut state = fresh_state();
    let tx = Transaction::transfer([1u8; 20], [2u8; 20], 50, 21_000, 0);

    // Act
    apply::<()>(
        &mut state,
        vec![Action::UpdateCache {
            update: CacheUpdate::AddPending(tx.clone()),
        }],
    );

    // Assert
    let txs = state.query_pending();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], tx);
}

#[test]
fn apply_mutations_broadcast_message_action_is_silently_ignored() {
    // Arrange
    let mut state = fresh_state();

    // Act
    apply(&mut state, vec![Action::BroadcastMessage { message: () }]);

    // Assert
    assert_eq!(state.query_height(), 0);
    assert!(state.query_pending().is_empty());
}
