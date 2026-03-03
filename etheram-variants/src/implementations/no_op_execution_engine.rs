// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::execution::execution_result::ExecutionResult;

pub struct NoOpExecutionEngine;

impl ExecutionEngine for NoOpExecutionEngine {
    fn execute(
        &self,
        _block: &Block,
        _accounts: &BTreeMap<Address, Account>,
        _contract_storage: &BTreeMap<(Address, Hash), Hash>,
    ) -> ExecutionResult {
        ExecutionResult {
            transaction_results: Vec::new(),
        }
    }
}
