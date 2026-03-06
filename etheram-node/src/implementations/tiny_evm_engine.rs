// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::transaction::Transaction;
use crate::common_types::types::Address;
use crate::common_types::types::Balance;
use crate::common_types::types::Gas;
use crate::common_types::types::Hash;
use crate::execution::execution_engine::ExecutionEngine;
use crate::execution::execution_result::ExecutionResult;
use crate::execution::transaction_result::TransactionResult;
use crate::execution::transaction_result::TransactionStatus;
use crate::implementations::tiny_evm_gas::memory_expansion_cost;
use crate::implementations::tiny_evm_gas::GAS_ADD;
use crate::implementations::tiny_evm_gas::GAS_AND;
use crate::implementations::tiny_evm_gas::GAS_CALLDATALOAD;
use crate::implementations::tiny_evm_gas::GAS_CALLDATASIZE;
use crate::implementations::tiny_evm_gas::GAS_CALLER;
use crate::implementations::tiny_evm_gas::GAS_CALLVALUE;
use crate::implementations::tiny_evm_gas::GAS_DIV;
use crate::implementations::tiny_evm_gas::GAS_DUP;
use crate::implementations::tiny_evm_gas::GAS_EQ;
use crate::implementations::tiny_evm_gas::GAS_GT;
use crate::implementations::tiny_evm_gas::GAS_ISZERO;
use crate::implementations::tiny_evm_gas::GAS_JUMP;
use crate::implementations::tiny_evm_gas::GAS_JUMPDEST;
use crate::implementations::tiny_evm_gas::GAS_JUMPI;
use crate::implementations::tiny_evm_gas::GAS_LT;
use crate::implementations::tiny_evm_gas::GAS_MLOAD_BASE;
use crate::implementations::tiny_evm_gas::GAS_MSTORE_BASE;
use crate::implementations::tiny_evm_gas::GAS_MUL;
use crate::implementations::tiny_evm_gas::GAS_OR;
use crate::implementations::tiny_evm_gas::GAS_POP;
use crate::implementations::tiny_evm_gas::GAS_PUSH1;
use crate::implementations::tiny_evm_gas::GAS_PUSHN;
use crate::implementations::tiny_evm_gas::GAS_RETURN;
use crate::implementations::tiny_evm_gas::GAS_REVERT;
use crate::implementations::tiny_evm_gas::GAS_SHA3_BASE;
use crate::implementations::tiny_evm_gas::GAS_SHA3_WORD;
use crate::implementations::tiny_evm_gas::GAS_SLOAD;
use crate::implementations::tiny_evm_gas::GAS_SSTORE_RESET;
use crate::implementations::tiny_evm_gas::GAS_SSTORE_SET;
use crate::implementations::tiny_evm_gas::GAS_STOP;
use crate::implementations::tiny_evm_gas::GAS_SUB;
use crate::implementations::tiny_evm_gas::GAS_SWAP;
use crate::implementations::tiny_evm_gas::INTRINSIC_GAS;
use crate::implementations::tiny_evm_opcode::decode_opcode;
pub use crate::implementations::tiny_evm_opcode::opcode_name;
use crate::implementations::tiny_evm_opcode::TinyEvmOpcode;
pub use crate::implementations::tiny_evm_opcode::OPCODE_ADD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_AND;
pub use crate::implementations::tiny_evm_opcode::OPCODE_CALLDATALOAD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_CALLDATASIZE;
pub use crate::implementations::tiny_evm_opcode::OPCODE_CALLER;
pub use crate::implementations::tiny_evm_opcode::OPCODE_CALLVALUE;
pub use crate::implementations::tiny_evm_opcode::OPCODE_DIV;
pub use crate::implementations::tiny_evm_opcode::OPCODE_DUP1;
pub use crate::implementations::tiny_evm_opcode::OPCODE_DUP16;
pub use crate::implementations::tiny_evm_opcode::OPCODE_EQ;
pub use crate::implementations::tiny_evm_opcode::OPCODE_GT;
pub use crate::implementations::tiny_evm_opcode::OPCODE_ISZERO;
pub use crate::implementations::tiny_evm_opcode::OPCODE_JUMP;
pub use crate::implementations::tiny_evm_opcode::OPCODE_JUMPDEST;
pub use crate::implementations::tiny_evm_opcode::OPCODE_JUMPI;
pub use crate::implementations::tiny_evm_opcode::OPCODE_LT;
pub use crate::implementations::tiny_evm_opcode::OPCODE_MLOAD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_MSTORE;
pub use crate::implementations::tiny_evm_opcode::OPCODE_MUL;
pub use crate::implementations::tiny_evm_opcode::OPCODE_OR;
pub use crate::implementations::tiny_evm_opcode::OPCODE_POP;
pub use crate::implementations::tiny_evm_opcode::OPCODE_PUSH1;
pub use crate::implementations::tiny_evm_opcode::OPCODE_PUSH2;
pub use crate::implementations::tiny_evm_opcode::OPCODE_PUSH32;
pub use crate::implementations::tiny_evm_opcode::OPCODE_RETURN;
pub use crate::implementations::tiny_evm_opcode::OPCODE_REVERT;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SHA3;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SLOAD;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SSTORE;
pub use crate::implementations::tiny_evm_opcode::OPCODE_STOP;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SUB;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SWAP1;
pub use crate::implementations::tiny_evm_opcode::OPCODE_SWAP16;
use crate::implementations::value_transfer::apply_value_transfers;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::slice::from_ref;
use primitive_types::U256;
use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

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
                    transaction,
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
                    TransactionStatus::OutOfGas => {
                        transaction_results.push(TransactionResult {
                            from: transaction.from,
                            status: TransactionStatus::OutOfGas,
                            gas_used: transaction.gas_limit,
                            mutations: Vec::new(),
                        });
                    }
                    TransactionStatus::Reverted => {
                        transaction_results.push(TransactionResult {
                            from: transaction.from,
                            status: TransactionStatus::Reverted,
                            gas_used: INTRINSIC_GAS + (gas_after_intrinsic - gas_left),
                            mutations: Vec::new(),
                        });
                    }
                    TransactionStatus::InvalidOpcode => {
                        transaction_results.push(TransactionResult {
                            from: transaction.from,
                            status: TransactionStatus::InvalidOpcode,
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
        TinyEvmOpcode::Mul => GAS_MUL,
        TinyEvmOpcode::Sub => GAS_SUB,
        TinyEvmOpcode::Div => GAS_DIV,
        TinyEvmOpcode::Lt => GAS_LT,
        TinyEvmOpcode::Gt => GAS_GT,
        TinyEvmOpcode::Eq => GAS_EQ,
        TinyEvmOpcode::IsZero => GAS_ISZERO,
        TinyEvmOpcode::And => GAS_AND,
        TinyEvmOpcode::Or => GAS_OR,
        TinyEvmOpcode::Sha3 => GAS_SHA3_BASE,
        TinyEvmOpcode::Caller => GAS_CALLER,
        TinyEvmOpcode::CallValue => GAS_CALLVALUE,
        TinyEvmOpcode::CallDataLoad => GAS_CALLDATALOAD,
        TinyEvmOpcode::CallDataSize => GAS_CALLDATASIZE,
        TinyEvmOpcode::Pop => GAS_POP,
        TinyEvmOpcode::Mload => GAS_MLOAD_BASE,
        TinyEvmOpcode::Mstore => GAS_MSTORE_BASE,
        TinyEvmOpcode::Sload => GAS_SLOAD,
        TinyEvmOpcode::Sstore => {
            let slot = stack.last().copied().unwrap_or([0u8; 32]);
            if local_storage.contains_key(&(contract_address, slot)) {
                GAS_SSTORE_RESET
            } else {
                GAS_SSTORE_SET
            }
        }
        TinyEvmOpcode::Jump => GAS_JUMP,
        TinyEvmOpcode::JumpI => GAS_JUMPI,
        TinyEvmOpcode::JumpDest => GAS_JUMPDEST,
        TinyEvmOpcode::Push(1) => GAS_PUSH1,
        TinyEvmOpcode::Push(_) => GAS_PUSHN,
        TinyEvmOpcode::Dup(_) => GAS_DUP,
        TinyEvmOpcode::Swap(_) => GAS_SWAP,
        TinyEvmOpcode::Return => GAS_RETURN,
        TinyEvmOpcode::Revert => GAS_REVERT,
    }
}

fn precompute_jumpdests(bytecode: &[u8]) -> BTreeSet<usize> {
    let mut jumpdests = BTreeSet::new();
    let mut i = 0;
    while i < bytecode.len() {
        let byte = bytecode[i];
        if byte == OPCODE_JUMPDEST {
            jumpdests.insert(i);
        } else if (OPCODE_PUSH1..=OPCODE_PUSH32).contains(&byte) {
            let n = (byte - OPCODE_PUSH1 + 1) as usize;
            i += n;
        }
        i += 1;
    }
    jumpdests
}

fn exec_add(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_from_u256(word_to_u256(&left).overflowing_add(word_to_u256(&right)).0);
    stack.push(result);
}

fn exec_mul(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_from_u256(word_to_u256(&left).overflowing_mul(word_to_u256(&right)).0);
    stack.push(result);
}

fn exec_sub(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_from_u256(word_to_u256(&left).overflowing_sub(word_to_u256(&right)).0);
    stack.push(result);
}

fn exec_div(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let divisor = word_to_u256(&right);
    let result = if divisor.is_zero() {
        U256::zero()
    } else {
        word_to_u256(&left) / divisor
    };
    stack.push(word_from_u256(result));
}

fn exec_lt(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = if word_to_u256(&left) < word_to_u256(&right) {
        U256::one()
    } else {
        U256::zero()
    };
    stack.push(word_from_u256(result));
}

fn exec_gt(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = if word_to_u256(&left) > word_to_u256(&right) {
        U256::one()
    } else {
        U256::zero()
    };
    stack.push(word_from_u256(result));
}

fn exec_eq(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = if left == right {
        U256::one()
    } else {
        U256::zero()
    };
    stack.push(word_from_u256(result));
}

fn exec_iszero(stack: &mut Vec<Hash>) {
    let val = pop_word(stack);
    let result = if word_to_u256(&val).is_zero() {
        U256::one()
    } else {
        U256::zero()
    };
    stack.push(word_from_u256(result));
}

fn exec_and(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_to_u256(&left) & word_to_u256(&right);
    stack.push(word_from_u256(result));
}

fn exec_or(stack: &mut Vec<Hash>) {
    let left = pop_word(stack);
    let right = pop_word(stack);
    let result = word_to_u256(&left) | word_to_u256(&right);
    stack.push(word_from_u256(result));
}

fn exec_sha3(
    stack: &mut Vec<Hash>,
    memory: &mut Vec<u8>,
    memory_words: &mut u64,
    gas: &mut Gas,
) -> bool {
    let offset = match word_to_u64(&pop_word(stack)) {
        Some(value) => value,
        None => return false,
    };
    let size = match word_to_u64(&pop_word(stack)) {
        Some(value) => value,
        None => return false,
    };
    let end_u64 = match offset.checked_add(size) {
        Some(value) => value,
        None => return false,
    };
    if end_u64 > usize::MAX as u64 || size > usize::MAX as u64 {
        return false;
    }
    let required_words = end_u64.div_ceil(32);
    if required_words > *memory_words {
        let expansion = memory_expansion_cost(*memory_words, required_words);
        if *gas < expansion {
            return false;
        }
        *gas -= expansion;
        *memory_words = required_words;
        let required_len = (required_words * 32) as usize;
        if memory.len() < required_len {
            memory.resize(required_len, 0);
        }
    }
    let words_for_sha = size.div_ceil(32);
    let sha3_extra = GAS_SHA3_WORD * words_for_sha;
    if *gas < sha3_extra {
        return false;
    }
    *gas -= sha3_extra;
    let start_usize = offset as usize;
    let size_usize = size as usize;
    let data: Vec<u8> = if start_usize < memory.len() {
        let available = memory.len() - start_usize;
        let take = available.min(size_usize);
        let mut buf = memory[start_usize..start_usize + take].to_vec();
        buf.resize(size_usize, 0);
        buf
    } else {
        alloc::vec![0u8; size_usize]
    };
    let mut output = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(&data);
    keccak.finalize(&mut output);
    stack.push(output);
    true
}

fn exec_caller(stack: &mut Vec<Hash>, from: Address) {
    let mut word = [0u8; 32];
    word[12..32].copy_from_slice(&from);
    stack.push(word);
}

fn exec_callvalue(stack: &mut Vec<Hash>, value: Balance) {
    let mut word = [0u8; 32];
    word[16..32].copy_from_slice(&value.to_be_bytes());
    stack.push(word);
}

fn exec_calldatasize(stack: &mut Vec<Hash>, calldata: &[u8]) {
    stack.push(word_from_u128(calldata.len() as u128));
}

fn exec_calldataload(stack: &mut Vec<Hash>, calldata: &[u8]) {
    let offset = match word_to_u64(&pop_word(stack)) {
        Some(value) if value <= usize::MAX as u64 => value as usize,
        _ => {
            stack.push([0u8; 32]);
            return;
        }
    };
    let mut word = [0u8; 32];
    for (i, b) in word.iter_mut().enumerate() {
        let src = offset + i;
        if src < calldata.len() {
            *b = calldata[src];
        }
    }
    stack.push(word);
}

fn exec_mstore(
    stack: &mut Vec<Hash>,
    memory: &mut Vec<u8>,
    memory_words: &mut u64,
    gas: &mut Gas,
) -> bool {
    let offset = match word_to_u64(&pop_word(stack)) {
        Some(value) => value,
        None => return false,
    };
    let value = pop_word(stack);
    let end_u64 = match offset.checked_add(32) {
        Some(value) => value,
        None => return false,
    };
    if end_u64 > usize::MAX as u64 {
        return false;
    }
    let required_words = end_u64.div_ceil(32);
    if required_words > *memory_words {
        let expansion = memory_expansion_cost(*memory_words, required_words);
        if *gas < expansion {
            return false;
        }
        *gas -= expansion;
        *memory_words = required_words;
    }
    let required_len = (*memory_words * 32) as usize;
    if memory.len() < required_len {
        memory.resize(required_len, 0);
    }
    let start = offset as usize;
    memory[start..start + 32].copy_from_slice(&value);
    true
}

fn exec_mload(
    stack: &mut Vec<Hash>,
    memory: &mut Vec<u8>,
    memory_words: &mut u64,
    gas: &mut Gas,
) -> bool {
    let offset = match word_to_u64(&pop_word(stack)) {
        Some(value) => value,
        None => return false,
    };
    let end_u64 = match offset.checked_add(32) {
        Some(value) => value,
        None => return false,
    };
    if end_u64 > usize::MAX as u64 {
        return false;
    }
    let required_words = end_u64.div_ceil(32);
    if required_words > *memory_words {
        let expansion = memory_expansion_cost(*memory_words, required_words);
        if *gas < expansion {
            return false;
        }
        *gas -= expansion;
        *memory_words = required_words;
    }
    let required_len = (*memory_words * 32) as usize;
    if memory.len() < required_len {
        memory.resize(required_len, 0);
    }
    let start = offset as usize;
    let mut word = [0u8; 32];
    word.copy_from_slice(&memory[start..start + 32]);
    stack.push(word);
    true
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

fn exec_push_n(stack: &mut Vec<Hash>, bytecode: &[u8], pc: &mut usize, n: usize) -> bool {
    let mut word = [0u8; 32];
    let start = 32 - n;
    for i in 0..n {
        if *pc >= bytecode.len() {
            return false;
        }
        word[start + i] = bytecode[*pc];
        *pc += 1;
    }
    stack.push(word);
    true
}

fn exec_dup(stack: &mut Vec<Hash>, n: usize) -> bool {
    if stack.len() < n {
        return false;
    }
    let val = stack[stack.len() - n];
    stack.push(val);
    true
}

fn exec_swap(stack: &mut [Hash], n: usize) -> bool {
    let top = stack.len();
    if top <= n {
        return false;
    }
    stack.swap(top - 1, top - 1 - n);
    true
}

fn execute_bytecode(
    bytecode: &[u8],
    contract_address: Address,
    contract_storage: &BTreeMap<(Address, Hash), Hash>,
    gas_remaining: Gas,
    transaction: &Transaction,
) -> (TransactionStatus, Gas, Vec<StorageMutation>) {
    let jumpdests = precompute_jumpdests(bytecode);
    let mut local_storage = contract_storage.clone();
    let mut mutations = Vec::new();
    let mut stack: Vec<Hash> = Vec::new();
    let mut memory: Vec<u8> = Vec::new();
    let mut memory_words: u64 = 0;
    let mut pc = 0usize;
    let mut gas = gas_remaining;
    while pc < bytecode.len() {
        let opcode_byte = bytecode[pc];
        pc += 1;
        let Some(opcode) = decode_opcode(opcode_byte) else {
            return (TransactionStatus::InvalidOpcode, 0, Vec::new());
        };
        let base_cost = opcode_gas_cost(&opcode, &stack, &local_storage, contract_address);
        if gas < base_cost {
            return (TransactionStatus::OutOfGas, 0, Vec::new());
        }
        gas -= base_cost;
        match opcode {
            TinyEvmOpcode::Stop => break,
            TinyEvmOpcode::Add => exec_add(&mut stack),
            TinyEvmOpcode::Mul => exec_mul(&mut stack),
            TinyEvmOpcode::Sub => exec_sub(&mut stack),
            TinyEvmOpcode::Div => exec_div(&mut stack),
            TinyEvmOpcode::Lt => exec_lt(&mut stack),
            TinyEvmOpcode::Gt => exec_gt(&mut stack),
            TinyEvmOpcode::Eq => exec_eq(&mut stack),
            TinyEvmOpcode::IsZero => exec_iszero(&mut stack),
            TinyEvmOpcode::And => exec_and(&mut stack),
            TinyEvmOpcode::Or => exec_or(&mut stack),
            TinyEvmOpcode::Sha3 => {
                if !exec_sha3(&mut stack, &mut memory, &mut memory_words, &mut gas) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
            }
            TinyEvmOpcode::Caller => exec_caller(&mut stack, transaction.from),
            TinyEvmOpcode::CallValue => exec_callvalue(&mut stack, transaction.value),
            TinyEvmOpcode::CallDataLoad => exec_calldataload(&mut stack, &transaction.data),
            TinyEvmOpcode::CallDataSize => exec_calldatasize(&mut stack, &transaction.data),
            TinyEvmOpcode::Pop => {
                pop_word(&mut stack);
            }
            TinyEvmOpcode::Mload => {
                if !exec_mload(&mut stack, &mut memory, &mut memory_words, &mut gas) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
            }
            TinyEvmOpcode::Mstore => {
                if !exec_mstore(&mut stack, &mut memory, &mut memory_words, &mut gas) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
            }
            TinyEvmOpcode::Sload => exec_sload(&mut stack, &local_storage, contract_address),
            TinyEvmOpcode::Sstore => exec_sstore(
                &mut stack,
                &mut local_storage,
                &mut mutations,
                contract_address,
            ),
            TinyEvmOpcode::Jump => {
                let dest = match word_to_u64(&pop_word(&mut stack)) {
                    Some(value) if value <= usize::MAX as u64 => value as usize,
                    _ => return (TransactionStatus::OutOfGas, 0, Vec::new()),
                };
                if !jumpdests.contains(&dest) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
                pc = dest;
            }
            TinyEvmOpcode::JumpI => {
                let dest = match word_to_u64(&pop_word(&mut stack)) {
                    Some(value) if value <= usize::MAX as u64 => value as usize,
                    _ => return (TransactionStatus::OutOfGas, 0, Vec::new()),
                };
                let cond = word_to_u256(&pop_word(&mut stack));
                if !cond.is_zero() {
                    if !jumpdests.contains(&dest) {
                        return (TransactionStatus::OutOfGas, 0, Vec::new());
                    }
                    pc = dest;
                }
            }
            TinyEvmOpcode::JumpDest => {}
            TinyEvmOpcode::Push(n) => {
                if !exec_push_n(&mut stack, bytecode, &mut pc, n as usize) {
                    break;
                }
            }
            TinyEvmOpcode::Dup(n) => {
                if !exec_dup(&mut stack, n as usize) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
            }
            TinyEvmOpcode::Swap(n) => {
                if !exec_swap(&mut stack, n as usize) {
                    return (TransactionStatus::OutOfGas, 0, Vec::new());
                }
            }
            TinyEvmOpcode::Return => break,
            TinyEvmOpcode::Revert => return (TransactionStatus::Reverted, gas, Vec::new()),
        }
    }
    (TransactionStatus::Success, gas, mutations)
}

fn pop_word(stack: &mut Vec<Hash>) -> Hash {
    stack.pop().unwrap_or([0u8; 32])
}

fn word_from_u128(value: u128) -> Hash {
    let mut word = [0u8; 32];
    word[16..32].copy_from_slice(&value.to_be_bytes());
    word
}

fn word_from_u256(value: U256) -> Hash {
    let mut word = [0u8; 32];
    value.to_big_endian(&mut word);
    word
}

fn word_to_u256(word: &Hash) -> U256 {
    U256::from_big_endian(word)
}

fn word_to_u64(word: &Hash) -> Option<u64> {
    let value = word_to_u256(word);
    if value > U256::from(u64::MAX) {
        return None;
    }
    Some(value.as_u64())
}
