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
    Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 0)
}

fn tx_b() -> Transaction {
    Transaction::transfer([3u8; 20], [4u8; 20], 200, 21_000, 1)
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
