// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::transaction::Transaction;
use crate::common_types::types::Address;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub fn apply_value_transfers(
    transactions: &[Transaction],
    accounts: &BTreeMap<Address, Account>,
) -> (BTreeMap<Address, Account>, Vec<StorageMutation>) {
    let mut working_accounts = accounts.clone();
    let mut mutations = Vec::new();
    for transaction in transactions {
        let from_account = working_accounts
            .get(&transaction.from)
            .cloned()
            .unwrap_or(Account::empty());
        let updated_from = Account {
            balance: from_account.balance.saturating_sub(transaction.value),
            nonce: from_account.nonce + 1,
        };
        working_accounts.insert(transaction.from, updated_from.clone());
        mutations.push(StorageMutation::UpdateAccount(
            transaction.from,
            updated_from,
        ));
        let to_account = working_accounts
            .get(&transaction.to)
            .cloned()
            .unwrap_or(Account::empty());
        let updated_to = Account {
            balance: to_account.balance.saturating_add(transaction.value),
            nonce: to_account.nonce,
        };
        working_accounts.insert(transaction.to, updated_to.clone());
        mutations.push(StorageMutation::UpdateAccount(transaction.to, updated_to));
    }
    (working_accounts, mutations)
}
