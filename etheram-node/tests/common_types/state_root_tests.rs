// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::account::Account;
use etheram_node::common_types::state_root::{
    compute_state_root, compute_state_root_with_contract_storage,
};
use etheram_node::common_types::types::{Address, Hash};
use std::collections::BTreeMap;

#[test]
fn compute_state_root_empty_accounts_returns_zero_hash() {
    // Arrange
    let accounts = BTreeMap::new();

    // Act
    let root = compute_state_root(&accounts);

    // Assert
    assert_eq!(root, [0u8; 32]);
}

#[test]
fn compute_state_root_same_accounts_returns_same_hash() {
    // Arrange
    let mut accounts = BTreeMap::new();
    accounts.insert([1u8; 20], Account::new(1000));

    // Act
    let first = compute_state_root(&accounts);
    let second = compute_state_root(&accounts);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn compute_state_root_different_balance_returns_different_hash() {
    // Arrange
    let address = [1u8; 20];
    let mut accounts_a = BTreeMap::new();
    accounts_a.insert(address, Account::new(100));
    let mut accounts_b = BTreeMap::new();
    accounts_b.insert(address, Account::new(200));

    // Act
    let root_a = compute_state_root(&accounts_a);
    let root_b = compute_state_root(&accounts_b);

    // Assert
    assert_ne!(root_a, root_b);
}

#[test]
fn compute_state_root_different_nonce_returns_different_hash() {
    // Arrange
    let address = [1u8; 20];
    let mut accounts_a = BTreeMap::new();
    accounts_a.insert(
        address,
        Account {
            balance: 500,
            nonce: 0,
        },
    );
    let mut accounts_b = BTreeMap::new();
    accounts_b.insert(
        address,
        Account {
            balance: 500,
            nonce: 1,
        },
    );

    // Act
    let root_a = compute_state_root(&accounts_a);
    let root_b = compute_state_root(&accounts_b);

    // Assert
    assert_ne!(root_a, root_b);
}

#[test]
fn compute_state_root_order_independent_returns_same_hash() {
    // Arrange
    let addr_a = [1u8; 20];
    let addr_b = [2u8; 20];
    let mut accounts_ab = BTreeMap::new();
    accounts_ab.insert(addr_a, Account::new(100));
    accounts_ab.insert(addr_b, Account::new(200));
    let mut accounts_ba = BTreeMap::new();
    accounts_ba.insert(addr_b, Account::new(200));
    accounts_ba.insert(addr_a, Account::new(100));

    // Act
    let root_ab = compute_state_root(&accounts_ab);
    let root_ba = compute_state_root(&accounts_ba);

    // Assert
    assert_eq!(root_ab, root_ba);
}

#[test]
fn compute_state_root_with_contract_storage_different_value_returns_different_hash() {
    // Arrange
    let accounts = BTreeMap::new();
    let address: Address = [5u8; 20];
    let slot: Hash = [6u8; 32];
    let mut contract_storage_a = BTreeMap::new();
    contract_storage_a.insert((address, slot), [7u8; 32]);
    let mut contract_storage_b = BTreeMap::new();
    contract_storage_b.insert((address, slot), [8u8; 32]);

    // Act
    let root_a = compute_state_root_with_contract_storage(&accounts, &contract_storage_a);
    let root_b = compute_state_root_with_contract_storage(&accounts, &contract_storage_b);

    // Assert
    assert_ne!(root_a, root_b);
}

#[test]
fn compute_state_root_with_contract_storage_order_independent_returns_same_hash() {
    // Arrange
    let accounts = BTreeMap::new();
    let address_a: Address = [9u8; 20];
    let address_b: Address = [10u8; 20];
    let slot_a: Hash = [11u8; 32];
    let slot_b: Hash = [12u8; 32];
    let mut contract_storage_ab = BTreeMap::new();
    contract_storage_ab.insert((address_a, slot_a), [13u8; 32]);
    contract_storage_ab.insert((address_b, slot_b), [14u8; 32]);
    let mut contract_storage_ba = BTreeMap::new();
    contract_storage_ba.insert((address_b, slot_b), [14u8; 32]);
    contract_storage_ba.insert((address_a, slot_a), [13u8; 32]);

    // Act
    let root_ab = compute_state_root_with_contract_storage(&accounts, &contract_storage_ab);
    let root_ba = compute_state_root_with_contract_storage(&accounts, &contract_storage_ba);

    // Assert
    assert_eq!(root_ab, root_ba);
}
