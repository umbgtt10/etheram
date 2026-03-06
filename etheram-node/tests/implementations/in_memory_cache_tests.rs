// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::cache::Cache;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::in_memory_cache::InMemoryCache;
use etheram_node::state::cache::cache_query::CacheQuery;
use etheram_node::state::cache::cache_query_result::CacheQueryResult;
use etheram_node::state::cache::cache_update::CacheUpdate;

fn tx_a() -> Transaction {
    Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 1, 0)
}

fn tx_b() -> Transaction {
    Transaction::transfer([3u8; 20], [4u8; 20], 200, 21_000, 1, 1)
}

#[test]
fn query_get_pending_empty_cache_returns_empty() {
    // Arrange
    let cache = InMemoryCache::new();

    // Act
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert!(txs.is_empty());
}

#[test]
fn update_add_pending_get_pending_returns_transaction() {
    // Arrange
    let mut cache = InMemoryCache::new();

    // Act
    cache.update(CacheUpdate::AddPending(tx_a()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], tx_a());
}

#[test]
fn update_add_pending_twice_get_pending_returns_both() {
    // Arrange
    let mut cache = InMemoryCache::new();

    // Act
    cache.update(CacheUpdate::AddPending(tx_a()));
    cache.update(CacheUpdate::AddPending(tx_b()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0], tx_a());
    assert_eq!(txs[1], tx_b());
}

#[test]
fn update_remove_pending_existing_transaction_removes_it() {
    // Arrange
    let mut cache = InMemoryCache::new();
    cache.update(CacheUpdate::AddPending(tx_a()));
    cache.update(CacheUpdate::AddPending(tx_b()));

    // Act
    cache.update(CacheUpdate::RemovePending(tx_a()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], tx_b());
}

#[test]
fn update_remove_pending_nonexistent_tx_leaves_others_unchanged() {
    // Arrange
    let mut cache = InMemoryCache::new();
    cache.update(CacheUpdate::AddPending(tx_a()));

    // Act
    cache.update(CacheUpdate::RemovePending(tx_b()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], tx_a());
}

#[test]
fn update_clear_pending_removes_all() {
    // Arrange
    let mut cache = InMemoryCache::new();
    cache.update(CacheUpdate::AddPending(tx_a()));
    cache.update(CacheUpdate::AddPending(tx_b()));

    // Act
    cache.update(CacheUpdate::ClearPending);
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert!(txs.is_empty());
}

#[test]
fn add_pending_orders_by_gas_price_descending() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let low = Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 5, 0);
    let high = Transaction::transfer([3u8; 20], [4u8; 20], 100, 21_000, 10, 0);

    // Act
    cache.update(CacheUpdate::AddPending(low.clone()));
    cache.update(CacheUpdate::AddPending(high.clone()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0], high);
    assert_eq!(txs[1], low);
}

#[test]
fn add_pending_per_sender_deduplication_replaces_on_higher_gas_price() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let from = [1u8; 20];
    let first = Transaction::transfer(from, [2u8; 20], 100, 21_000, 5, 0);
    let higher = Transaction::transfer(from, [2u8; 20], 100, 21_000, 10, 0);

    // Act
    cache.update(CacheUpdate::AddPending(first));
    cache.update(CacheUpdate::AddPending(higher.clone()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], higher);
}

#[test]
fn add_pending_per_sender_deduplication_rejects_equal_gas_price() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let from = [1u8; 20];
    let first = Transaction::transfer(from, [2u8; 20], 100, 21_000, 5, 0);
    let equal = Transaction::transfer(from, [3u8; 20], 100, 21_000, 5, 0);

    // Act
    cache.update(CacheUpdate::AddPending(first.clone()));
    cache.update(CacheUpdate::AddPending(equal));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0], first);
}

#[test]
fn get_pending_returns_descending_gas_price_order() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let low = Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 1, 0);
    let mid = Transaction::transfer([3u8; 20], [4u8; 20], 100, 21_000, 5, 0);
    let high = Transaction::transfer([5u8; 20], [6u8; 20], 100, 21_000, 10, 0);

    // Act
    cache.update(CacheUpdate::AddPending(mid.clone()));
    cache.update(CacheUpdate::AddPending(low.clone()));
    cache.update(CacheUpdate::AddPending(high.clone()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), 3);
    assert_eq!(txs[0], high);
    assert_eq!(txs[1], mid);
    assert_eq!(txs[2], low);
}
