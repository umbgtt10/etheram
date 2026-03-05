// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::collections::action_collection::ActionCollection;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node::state::cache::cache_update::CacheUpdate;
use etheram_node::state::etheram_state::EtheramState;
use etheram_variants::implementations::eager_context_builder::EagerContextBuilder;
use etheram_variants::implementations::in_memory_cache::InMemoryCache;
use etheram_variants::implementations::in_memory_storage::InMemoryStorage;

fn state_with_storage(storage: InMemoryStorage) -> EtheramState {
    EtheramState::new(Box::new(storage), Box::new(InMemoryCache::new()))
}

fn neutral_message() -> Message<()> {
    Message::<()>::Timer(TimerEvent::ProposeBlock)
}

fn source() -> MessageSource {
    MessageSource::Timer
}

fn peer_id() -> u64 {
    0
}

#[test]
fn build_empty_state_neutral_message_returns_height_zero_and_empty_accounts() {
    // Arrange
    let state = state_with_storage(InMemoryStorage::new());
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(&state, peer_id(), &source(), &neutral_message());

    // Assert
    assert_eq!(ctx.current_height, 0);
    assert!(ctx.accounts.is_empty());
    assert!(ctx.pending_txs.is_empty());
}

#[test]
fn build_incremented_height_neutral_message_returns_updated_height() {
    // Arrange
    let mut state = state_with_storage(InMemoryStorage::new());
    state.apply_mutations(&ActionCollection::<Action<()>>::from_vec(vec![
        Action::IncrementHeight,
        Action::IncrementHeight,
    ]));
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(&state, peer_id(), &source(), &neutral_message());

    // Assert
    assert_eq!(ctx.current_height, 2);
}

#[test]
fn build_get_balance_known_address_includes_account() {
    // Arrange
    let address: Address = [1u8; 20];
    let state = state_with_storage(InMemoryStorage::new().with_genesis_account(address, 750));
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::GetBalance(address)),
    );

    // Assert
    assert_eq!(ctx.accounts.len(), 1);
    assert_eq!(ctx.accounts[&address].balance, 750);
}

#[test]
fn build_get_balance_unknown_address_returns_empty_accounts() {
    // Arrange
    let address: Address = [9u8; 20];
    let state = state_with_storage(InMemoryStorage::new());
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::GetBalance(address)),
    );

    // Assert
    assert!(ctx.accounts.is_empty());
}

#[test]
fn build_get_balance_zero_balance_zero_nonce_account_excluded() {
    // Arrange
    let address: Address = [2u8; 20];
    let mut state = state_with_storage(InMemoryStorage::new());
    state.apply_mutations(&ActionCollection::<Action<()>>::from_vec(vec![
        Action::UpdateAccount {
            address,
            account: Account::empty(),
        },
    ]));
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::GetBalance(address)),
    );

    // Assert
    assert!(ctx.accounts.is_empty());
}

#[test]
fn build_submit_transaction_from_address_with_balance_loaded_into_context() {
    // Arrange
    let from: Address = [3u8; 20];
    let to: Address = [4u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let state = state_with_storage(InMemoryStorage::new().with_genesis_account(from, 1000));
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::SubmitTransaction(tx)),
    );

    // Assert
    assert_eq!(ctx.accounts.len(), 1);
    assert_eq!(ctx.accounts[&from].balance, 1000);
}

#[test]
fn build_submit_transaction_unknown_from_address_returns_empty_accounts() {
    // Arrange
    let from: Address = [5u8; 20];
    let to: Address = [6u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let state = state_with_storage(InMemoryStorage::new());
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::SubmitTransaction(tx)),
    );

    // Assert
    assert!(ctx.accounts.is_empty());
}

#[test]
fn build_pending_txs_from_and_to_addresses_loaded_into_context() {
    // Arrange
    let from: Address = [7u8; 20];
    let to: Address = [8u8; 20];
    let tx = Transaction::transfer(from, to, 50, 21_000, 0);
    let mut state = state_with_storage(
        InMemoryStorage::new()
            .with_genesis_account(from, 500)
            .with_genesis_account(to, 200),
    );
    state.apply_mutations(&ActionCollection::<Action<()>>::from_vec(vec![
        Action::UpdateCache {
            update: CacheUpdate::AddPending(tx.clone()),
        },
    ]));
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(&state, peer_id(), &source(), &neutral_message());

    // Assert
    assert_eq!(ctx.pending_txs.len(), 1);
    assert_eq!(ctx.accounts.len(), 2);
    assert_eq!(ctx.accounts[&from].balance, 500);
    assert_eq!(ctx.accounts[&to].balance, 200);
}

#[test]
fn build_get_height_message_does_not_add_extra_accounts() {
    // Arrange
    let state = state_with_storage(InMemoryStorage::new());
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::GetHeight),
    );

    // Assert
    assert!(ctx.accounts.is_empty());
    assert_eq!(ctx.current_height, 0);
}

#[test]
fn build_submit_transaction_does_not_prefetch_receiver_address_into_context() {
    // Arrange
    let from: Address = [3u8; 20];
    let to: Address = [4u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let state = state_with_storage(
        InMemoryStorage::new()
            .with_genesis_account(from, 1_000)
            .with_genesis_account(to, 500),
    );
    let builder = EagerContextBuilder::new();

    // Act
    let ctx = builder.build(
        &state,
        peer_id(),
        &source(),
        &Message::<()>::Client(ClientRequest::SubmitTransaction(tx)),
    );

    // Assert
    assert_eq!(ctx.accounts.len(), 1);
    assert!(!ctx.accounts.contains_key(&to));
}
