// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::types::Address;
use crate::common_types::types::Gas;
use crate::common_types::types::Hash;
use crate::execution::execution_engine::ExecutionEngine;
use crate::execution::execution_result::ExecutionResult;
use crate::execution::transaction_result::TransactionResult;
use crate::execution::transaction_result::TransactionStatus;
use crate::implementations::tiny_evm_gas::GAS_ADD;
use crate::implementations::tiny_evm_gas::GAS_PUSH1;
use crate::implementations::tiny_evm_gas::GAS_RETURN;
use crate::implementations::tiny_evm_gas::GAS_SLOAD;
use crate::implementations::tiny_evm_gas::GAS_SSTORE_RESET;
use crate::implementations::tiny_evm_gas::GAS_SSTORE_SET;
use crate::implementations::tiny_evm_gas::GAS_STOP;
use crate::implementations::tiny_evm_gas::INTRINSIC_GAS;
use crate::implementations::tiny_evm_opcode::decode_opcode;
pub use crate::implementations::tiny_evm_opcode::opcode_name;
use crate::implementations::tiny_evm_opcode::TinyEvmOpcode;
pub use crate::implementations::tiny_evm_opcode::OPCODE_ADD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_PUSH1;
pub use crate::implementations::tiny_evm_opcode::OPCODE_RETURN;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SLOAD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SSTORE;
pub use crate::implementations::tiny_evm_opcode::OPCODE_STOP;
use crate::implementations::value_transfer::apply_value_transfers;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::slice::from_ref;

pub struct TinyEvmEngine;

impl ExecutionEngine for TinyEvmEngine {
    fn execute(
        &self,
        block: &Block,
        accounts: &BTreeMap<Address, Account>,
        contract_storage: &BTreeMap<(Address, Hash), Hash>,
    ) -> ExecutionResult {
        let mut working_accounts = accounts.clone();
        let mut working_contract_storage = contract_storage.clone();
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
            let (updated, value_mutations) =
                apply_value_transfers(from_ref(transaction), &working_accounts);
            let gas_after_intrinsic = transaction.gas_limit - INTRINSIC_GAS;
            if !transaction.data.is_empty() {
                let (status, gas_left, bytecode_mutations) = execute_bytecode(
                    &transaction.data,
                    transaction.to,
                    &working_contract_storage,
                    gas_after_intrinsic,
                );
                match status {
                    TransactionStatus::Success => {
                        for mutation in &bytecode_mutations {
                            if let StorageMutation::UpdateContractStorage {
                                address,
                                slot,
                                value,
                            } = mutation
                            {
                                working_contract_storage.insert((*address, *slot), *value);
                            }
                        }
                        working_accounts = updated;
                        let mut mutations = value_mutations;
                        mutations.extend(bytecode_mutations);
                        transaction_results.push(TransactionResult {
                            from: transaction.from,
                            status: TransactionStatus::Success,
                            gas_used: INTRINSIC_GAS + (gas_after_intrinsic - gas_left),
                            mutations,
                        });
                    }
                    _ => {
                        transaction_results.push(TransactionResult {
                            from: transaction.from,
                            status: TransactionStatus::OutOfGas,
                            gas_used: transaction.gas_limit,
                            mutations: Vec::new(),
                        });
                    }
                }
            } else {
                working_accounts = updated;
                transaction_results.push(TransactionResult {
                    from: transaction.from,
                    status: TransactionStatus::Success,
                    gas_used: INTRINSIC_GAS,
                    mutations: value_mutations,
                });
            }
        }
        ExecutionResult {
            transaction_results,
        }
    }
}

fn opcode_gas_cost(
    opcode: &TinyEvmOpcode,
    stack: &[Hash],
    local_storage: &BTreeMap<(Address, Hash), Hash>,
    contract_address: Address,
) -> Gas {
    match opcode {
        TinyEvmOpcode::Stop => GAS_STOP,
        TinyEvmOpcode::Add => GAS_ADD,
        TinyEvmOpcode::Sload => GAS_SLOAD,
        TinyEvmOpcode::Sstore => {
            let slot = stack.last().copied().unwrap_or([0u8; 32]);
            if local_storage.contains_key(&(contract_address, slot)) {
                GAS_SSTORE_RESET
            } else {
                GAS_SSTORE_SET
            }
        }
        TinyEvmOpcode::Push1 => GAS_PUSH1,
        TinyEvmOpcode::Return => GAS_RETURN,
    }
}

fn exec_add(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_from_u128(word_to_u128(&left).wrapping_add(word_to_u128(&right)));
    stack.push(result);
}

fn exec_sload(
    stack: &mut Vec<Hash>,
    local_storage: &BTreeMap<(Address, Hash), Hash>,
    contract_address: Address,
) {
    let slot = pop_word(stack);
    let value = local_storage
        .get(&(contract_address, slot))
        .copied()
        .unwrap_or([0u8; 32]);
    stack.push(value);
}

fn exec_sstore(
    stack: &mut Vec<Hash>,
    local_storage: &mut BTreeMap<(Address, Hash), Hash>,
    mutations: &mut Vec<StorageMutation>,
    contract_address: Address,
) {
    let slot = pop_word(stack);
    let value = pop_word(stack);
    local_storage.insert((contract_address, slot), value);
    mutations.push(StorageMutation::UpdateContractStorage {
        address: contract_address,
        slot,
        value,
    });
}

fn exec_push1(stack: &mut Vec<Hash>, bytecode: &[u8], pc: &mut usize) -> bool {
    if *pc >= bytecode.len() {
        return false;
    }
    let immediate = bytecode[*pc];
    *pc += 1;
    stack.push(word_from_u8(immediate));
    true
}

fn execute_bytecode(
    bytecode: &[u8],
    contract_address: Address,
    contract_storage: &BTreeMap<(Address, Hash), Hash>,
    gas_remaining: Gas,
) -> (TransactionStatus, Gas, Vec<StorageMutation>) {
    let mut local_storage = contract_storage.clone();
    let mut mutations = Vec::new();
    let mut stack: Vec<Hash> = Vec::new();
    let mut pc = 0usize;
    let mut gas = gas_remaining;
    while pc < bytecode.len() {
        let opcode_byte = bytecode[pc];
        pc += 1;
        let Some(opcode) = decode_opcode(opcode_byte) else {
            return (TransactionStatus::OutOfGas, 0, Vec::new());
        };
        let opcode_cost = opcode_gas_cost(&opcode, &stack, &local_storage, contract_address);
        if gas < opcode_cost {
            return (TransactionStatus::OutOfGas, 0, Vec::new());
        }
        gas -= opcode_cost;
        match opcode {
            TinyEvmOpcode::Stop => break,
            TinyEvmOpcode::Add => exec_add(&mut stack),
            TinyEvmOpcode::Sload => exec_sload(&mut stack, &local_storage, contract_address),
            TinyEvmOpcode::Sstore => exec_sstore(
                &mut stack,
                &mut local_storage,
                &mut mutations,
                contract_address,
            ),
            TinyEvmOpcode::Push1 => {
                if !exec_push1(&mut stack, bytecode, &mut pc) {
                    break;
                }
            }
            TinyEvmOpcode::Return => break,
        }
    }
    (TransactionStatus::Success, gas, mutations)
}

fn pop_word(stack: &mut Vec<Hash>) -> Hash {
    stack.pop().unwrap_or([0u8; 32])
}

fn word_from_u8(value: u8) -> Hash {
    let mut word = [0u8; 32];
    word[31] = value;
    word
}

fn word_from_u128(value: u128) -> Hash {
    let mut word = [0u8; 32];
    word[16..32].copy_from_slice(&value.to_be_bytes());
    word
}

fn word_to_u128(word: &Hash) -> u128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&word[16..32]);
    u128::from_be_bytes(bytes)
}
