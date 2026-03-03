// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::account::Account;
use super::types::{Address, Hash};
use alloc::collections::BTreeMap;

pub fn compute_state_root(accounts: &BTreeMap<Address, Account>) -> Hash {
    let contract_storage = BTreeMap::new();
    compute_state_root_with_contract_storage(accounts, &contract_storage)
}

pub fn compute_state_root_with_contract_storage(
    accounts: &BTreeMap<Address, Account>,
    contract_storage: &BTreeMap<(Address, Hash), Hash>,
) -> Hash {
    let mut hash = [0u8; 32];
    for (position, (addr, account)) in (0_u64..).zip(accounts.iter()) {
        let mix = position.wrapping_add(1) as u8;
        for i in 0..20 {
            hash[i % 32] ^= addr[i].wrapping_add(mix);
        }
        for (i, b) in account.balance.to_le_bytes().iter().enumerate() {
            hash[(i + 20) % 32] ^= b.wrapping_mul(mix);
        }
        for (i, b) in account.nonce.to_le_bytes().iter().enumerate() {
            hash[(i + 4) % 32] ^= b.wrapping_add(mix);
        }
    }
    for (position, ((address, slot), value)) in (0_u64..).zip(contract_storage.iter()) {
        let mix = position.wrapping_add(1) as u8;
        for (i, b) in address.iter().enumerate() {
            hash[(i + 7) % 32] ^= b.wrapping_add(mix);
        }
        for (i, b) in slot.iter().enumerate() {
            hash[(i + 13) % 32] ^= b.wrapping_mul(mix);
        }
        for (i, b) in value.iter().enumerate() {
            hash[(i + 19) % 32] ^= b.wrapping_add(mix);
        }
    }
    hash
}
