// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::types::Address;
use crate::common_types::types::Hash;
use crate::execution::execution_engine::ExecutionEngine;
use crate::execution::execution_result::ExecutionResult;
use crate::execution::transaction_result::TransactionResult;
use crate::execution::transaction_result::TransactionStatus;
use crate::implementations::tiny_evm_gas::INTRINSIC_GAS;
use crate::implementations::value_transfer::apply_value_transfers;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::slice::from_ref;

pub struct ValueTransferEngine;

impl ExecutionEngine for ValueTransferEngine {
    fn execute(
        &self,
        block: &Block,
        accounts: &BTreeMap<Address, Account>,
        _contract_storage: &BTreeMap<(Address, Hash), Hash>,
    ) -> ExecutionResult {
        let mut working_accounts = accounts.clone();
        let mut transaction_results = Vec::new();
        for transaction in &block.transactions {
            if transaction.gas_limit < INTRINSIC_GAS {
                transaction_results.push(TransactionResult {
                    from: transaction.from,
                    status: TransactionStatus::OutOfGas,
                    gas_used: transaction.gas_limit,
                    mutations: Vec::new(),
                });
                continue;
            }
            let (updated, mutations) =
                apply_value_transfers(from_ref(transaction), &working_accounts);
            working_accounts = updated;
            transaction_results.push(TransactionResult {
                from: transaction.from,
                status: TransactionStatus::Success,
                gas_used: INTRINSIC_GAS,
                mutations,
            });
        }
        ExecutionResult {
            transaction_results,
        }
    }
}
