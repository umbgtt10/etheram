// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::cache::Cache;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::in_memory_cache::InMemoryCache;
use etheram_node::implementations::in_memory_cache::PENDING_TX_POOL_CAPACITY;
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

#[test]
fn add_pending_when_pool_full_evicts_lowest_priority() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let from = [7u8; 20];
    for nonce in 0..PENDING_TX_POOL_CAPACITY as u64 {
        let tx = Transaction::transfer(from, [8u8; 20], 1, 21_000, 1, nonce);
        cache.update(CacheUpdate::AddPending(tx));
    }
    let evicted = Transaction::transfer(from, [8u8; 20], 1, 21_000, 1, 4095);
    let replacement = Transaction::transfer([9u8; 20], [8u8; 20], 1, 21_000, 2, 0);

    // Act
    cache.update(CacheUpdate::AddPending(replacement.clone()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs.len(), PENDING_TX_POOL_CAPACITY);
    assert_eq!(txs[0], replacement);
    assert!(!txs.contains(&evicted));
}

#[test]
fn get_pending_same_gas_price_orders_by_nonce_and_sender_tiebreakers() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let high_nonce = Transaction::transfer([2u8; 20], [3u8; 20], 1, 21_000, 5, 7);
    let low_nonce = Transaction::transfer([2u8; 20], [3u8; 20], 1, 21_000, 5, 1);
    let lower_sender = Transaction::transfer([1u8; 20], [3u8; 20], 1, 21_000, 5, 1);

    // Act
    cache.update(CacheUpdate::AddPending(high_nonce.clone()));
    cache.update(CacheUpdate::AddPending(low_nonce.clone()));
    cache.update(CacheUpdate::AddPending(lower_sender.clone()));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert_eq!(txs[0], lower_sender);
    assert_eq!(txs[1], low_nonce);
    assert_eq!(txs[2], high_nonce);
}

#[test]
fn remove_pending_same_sender_nonce_different_payload_removes_indexed_entry() {
    // Arrange
    let mut cache = InMemoryCache::new();
    let from = [1u8; 20];
    let inserted = Transaction::new(from, [2u8; 20], 10, 21_000, 9, 4, vec![1, 2, 3]);
    let different_payload_same_key =
        Transaction::new(from, [3u8; 20], 99, 30_000, 1, 4, vec![9, 9, 9]);
    cache.update(CacheUpdate::AddPending(inserted));

    // Act
    cache.update(CacheUpdate::RemovePending(different_payload_same_key));
    let CacheQueryResult::Pending(txs) = cache.query(CacheQuery::GetPending);

    // Assert
    assert!(txs.is_empty());
}
