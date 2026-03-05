// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use etheram_node::execution::transaction_receipt::TransactionReceipt;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::implementations::in_memory_storage::InMemoryStorage;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;

#[test]
fn store_receipts_and_query_returns_stored_receipts() {
    // Arrange
    let receipt = TransactionReceipt {
        status: TransactionStatus::Success,
        gas_used: 21_000,
        cumulative_gas_used: 21_000,
    };
    let mut storage = InMemoryStorage::new();

    // Act
    storage.mutate(StorageMutation::StoreReceipts(1, vec![receipt]));

    // Assert
    let StorageQueryResult::Receipts(result) = storage.query(StorageQuery::GetReceipts(1)) else {
        panic!("expected Receipts variant");
    };
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, TransactionStatus::Success);
    assert_eq!(result[0].gas_used, 21_000);
    assert_eq!(result[0].cumulative_gas_used, 21_000);
}

#[test]
fn store_receipts_multiple_heights_queries_correct_height() {
    // Arrange
    let r1 = TransactionReceipt {
        status: TransactionStatus::Success,
        gas_used: 21_000,
        cumulative_gas_used: 21_000,
    };
    let r2 = TransactionReceipt {
        status: TransactionStatus::OutOfGas,
        gas_used: 100,
        cumulative_gas_used: 100,
    };
    let mut storage = InMemoryStorage::new();
    storage.mutate(StorageMutation::StoreReceipts(0, vec![r1]));
    storage.mutate(StorageMutation::StoreReceipts(1, vec![r2]));

    // Act
    let StorageQueryResult::Receipts(result) = storage.query(StorageQuery::GetReceipts(1)) else {
        panic!("expected Receipts variant");
    };

    // Assert
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, TransactionStatus::OutOfGas);
    assert_eq!(result[0].gas_used, 100);
}

#[test]
fn query_receipts_unknown_height_returns_empty() {
    // Arrange
    let storage = InMemoryStorage::new();

    // Act
    let StorageQueryResult::Receipts(result) = storage.query(StorageQuery::GetReceipts(99)) else {
        panic!("expected Receipts variant");
    };

    // Assert
    assert!(result.is_empty());
}
