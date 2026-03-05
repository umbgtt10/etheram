// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::state_root::compute_state_root_with_contract_storage;
use crate::common_types::types::Address;
use crate::common_types::types::Hash;
use crate::execution::execution_engine::ExecutionEngine;
use crate::execution::receipts_root::compute_receipts_root;
use crate::execution::transaction_result::TransactionStatus;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::collections::BTreeMap;

pub fn compute_block_commitments(
    block: &Block,
    accounts: &BTreeMap<Address, Account>,
    contract_storage: &BTreeMap<(Address, Hash), Hash>,
    engine: &dyn ExecutionEngine,
) -> (Hash, Hash) {
    if block.transactions.is_empty() {
        return (block.state_root, compute_receipts_root(&[]));
    }
    let result = engine.execute(block, accounts, contract_storage);
    let mut post_accounts = accounts.clone();
    let mut post_storage = contract_storage.clone();
    for tx_result in &result.transaction_results {
        if tx_result.status == TransactionStatus::Success {
            for mutation in &tx_result.mutations {
                match mutation {
                    StorageMutation::UpdateAccount(addr, account) => {
                        post_accounts.insert(*addr, account.clone());
                    }
                    StorageMutation::UpdateContractStorage {
                        address,
                        slot,
                        value,
                    } => {
                        post_storage.insert((*address, *slot), *value);
                    }
                    _ => {}
                }
            }
        }
    }
    let post_state_root = compute_state_root_with_contract_storage(&post_accounts, &post_storage);
    let receipts_root = compute_receipts_root(&result.transaction_results);
    (post_state_root, receipts_root)
}
