// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::types::Address;
use crate::common_types::types::Hash;
use crate::execution::execution_result::ExecutionResult;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub trait ExecutionEngine {
    fn execute(
        &self,
        block: &Block,
        accounts: &BTreeMap<Address, Account>,
        contract_storage: &BTreeMap<(Address, Hash), Hash>,
    ) -> ExecutionResult;
}

pub type BoxedExecutionEngine = Box<dyn ExecutionEngine>;
