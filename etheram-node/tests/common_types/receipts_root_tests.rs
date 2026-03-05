// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::execution::receipts_root::compute_receipts_root;
use etheram_node::execution::transaction_result::TransactionResult;
use etheram_node::execution::transaction_result::TransactionStatus;

#[test]
fn compute_receipts_root_empty_results_returns_zero_hash() {
    // Arrange
    let results: Vec<TransactionResult> = vec![];

    // Act
    let root = compute_receipts_root(&results);

    // Assert
    assert_eq!(root, [0u8; 32]);
}

#[test]
fn compute_receipts_root_same_results_twice_returns_same_hash() {
    // Arrange
    let results = vec![
        TransactionResult {
            from: [1u8; 20],
            status: TransactionStatus::Success,
            gas_used: 21_000,
            mutations: vec![],
        },
        TransactionResult {
            from: [2u8; 20],
            status: TransactionStatus::OutOfGas,
            gas_used: 50_000,
            mutations: vec![],
        },
    ];

    // Act
    let first = compute_receipts_root(&results);
    let second = compute_receipts_root(&results);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn compute_receipts_root_different_status_returns_different_hash() {
    // Arrange
    let success = vec![TransactionResult {
        from: [1u8; 20],
        status: TransactionStatus::Success,
        gas_used: 21_000,
        mutations: vec![],
    }];
    let out_of_gas = vec![TransactionResult {
        from: [1u8; 20],
        status: TransactionStatus::OutOfGas,
        gas_used: 21_000,
        mutations: vec![],
    }];

    // Act
    let success_root = compute_receipts_root(&success);
    let out_of_gas_root = compute_receipts_root(&out_of_gas);

    // Assert
    assert_ne!(success_root, out_of_gas_root);
}

#[test]
fn compute_receipts_root_different_gas_used_returns_different_hash() {
    // Arrange
    let low_gas = vec![TransactionResult {
        from: [1u8; 20],
        status: TransactionStatus::Success,
        gas_used: 21_000,
        mutations: vec![],
    }];
    let high_gas = vec![TransactionResult {
        from: [1u8; 20],
        status: TransactionStatus::Success,
        gas_used: 50_000,
        mutations: vec![],
    }];

    // Act
    let low_root = compute_receipts_root(&low_gas);
    let high_root = compute_receipts_root(&high_gas);

    // Assert
    assert_ne!(low_root, high_root);
}

#[test]
fn compute_receipts_root_ordering_matters_returns_different_hash() {
    // Arrange
    let result_a = TransactionResult {
        from: [1u8; 20],
        status: TransactionStatus::Success,
        gas_used: 21_000,
        mutations: vec![],
    };
    let result_b = TransactionResult {
        from: [2u8; 20],
        status: TransactionStatus::OutOfGas,
        gas_used: 50_000,
        mutations: vec![],
    };
    let order_ab = vec![
        TransactionResult {
            from: result_a.from,
            status: result_a.status,
            gas_used: result_a.gas_used,
            mutations: vec![],
        },
        TransactionResult {
            from: result_b.from,
            status: result_b.status,
            gas_used: result_b.gas_used,
            mutations: vec![],
        },
    ];
    let order_ba = vec![
        TransactionResult {
            from: result_b.from,
            status: result_b.status,
            gas_used: result_b.gas_used,
            mutations: vec![],
        },
        TransactionResult {
            from: result_a.from,
            status: result_a.status,
            gas_used: result_a.gas_used,
            mutations: vec![],
        },
    ];

    // Act
    let root_ab = compute_receipts_root(&order_ab);
    let root_ba = compute_receipts_root(&order_ba);

    // Assert
    assert_ne!(root_ab, root_ba);
}
