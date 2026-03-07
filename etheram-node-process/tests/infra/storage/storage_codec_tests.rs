// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::account::Account;
use etheram_node::execution::transaction_receipt::TransactionReceipt;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node_process::infra::storage::storage_codec::StorageCodec;

#[test]
fn encode_account_decode_account_round_trips_exactly() {
    // Arrange
    let account = Account {
        balance: 777,
        nonce: 13,
    };
    let encoded = StorageCodec::encode_account(&account).expect("failed to encode account");

    // Act
    let decoded = StorageCodec::decode_account(&encoded).expect("failed to decode account");

    // Assert
    assert_eq!(decoded, account);
}

#[test]
fn encode_receipts_decode_receipts_round_trips_mixed_statuses() {
    // Arrange
    let receipts = vec![
        TransactionReceipt {
            status: TransactionStatus::Success,
            gas_used: 21_000,
            cumulative_gas_used: 21_000,
        },
        TransactionReceipt {
            status: TransactionStatus::OutOfGas,
            gas_used: 50_000,
            cumulative_gas_used: 71_000,
        },
        TransactionReceipt {
            status: TransactionStatus::Reverted,
            gas_used: 13_000,
            cumulative_gas_used: 84_000,
        },
        TransactionReceipt {
            status: TransactionStatus::InvalidOpcode,
            gas_used: 1_100,
            cumulative_gas_used: 85_100,
        },
    ];
    let encoded = StorageCodec::encode_receipts(&receipts).expect("failed to encode receipts");

    // Act
    let decoded = StorageCodec::decode_receipts(&encoded).expect("failed to decode receipts");

    // Assert
    assert_eq!(decoded, receipts);
}
