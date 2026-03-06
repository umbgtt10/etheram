# C-Functionalities Implementation Plan

This document is the detailed implementation plan for C3, C6, and C1 in order.
Each feature section describes every file that must change or be created, the exact
data model changes, the invariants that must be preserved, and the test obligations.

---

## C3 — Transaction Pool with Priority Ordering

### Goal

Replace the flat `Vec<Transaction>` in `InMemoryCache` with a priority-ordered pool
that sorts by descending gas price, enforces per-sender nonce sequencing, and caps the
pool at a configurable maximum size.

### Current state

`Transaction` carries `gas_limit` (a per-tx gas ceiling) but no `gas_price` field.
`InMemoryCache` holds a `Vec<Transaction>` and appends on `AddPending`. The proposer
takes `ctx.pending_txs` verbatim and stuffs the whole thing into the block.

### Data model changes

**`etheram-node/src/common_types/transaction.rs`**
- Add `pub gas_price: u64` field between `gas_limit` and `nonce`.
- Update `new(...)` and `transfer(...)` constructors — add `gas_price` parameter.
- `Transaction` derives `PartialOrd`/`Ord` keyed by `(gas_price DESC, nonce ASC,
  from ASC)` so the pool sort order is deterministic and canonical across nodes.

**`etheram-node/src/common_types/types.rs`**
- Add `pub type GasPrice = u64;` type alias.

### Pool capacity constant

**`etheram-node/src/implementations/in_memory_cache.rs`**
- Add `pub const PENDING_TX_POOL_CAPACITY: usize = 4096;` at the top of the file.

### `InMemoryCache` changes

**`etheram-node/src/implementations/in_memory_cache.rs`**
- Replace `pending_txs: Vec<Transaction>` with `pending_txs: BTreeSet<Transaction>`.
  `BTreeSet` maintains sorted order (ascending by `Ord`) at zero extra cost for
  a pool that is read at every proposal round.
- `AddPending(tx)`:
  - Reject silently if `pending_txs.len() >= PENDING_TX_POOL_CAPACITY` and `tx`
    would not displace the lowest-priority tx currently in the pool. If displacing,
    remove the lowest-priority tx first, then insert the new one.
  - Per-sender nonce duplicate check: if any tx with the same `(from, nonce)` is
    already in the pool, the new tx replaces it only when its `gas_price` is strictly
    higher (replacement at equal gas price is rejected to avoid churn).
- `RemovePending(tx)`: calls `pending_txs.remove(&tx)` — `O(log n)`.
- `ClearPending`: clears the set.
- `GetPending`: returns `pending_txs.iter().rev().cloned().collect::<Vec<_>>()` —
  highest gas price first, which is the order the proposer should use.

### Protocol changes

**`etheram-node/src/implementations/ibft/ibft_protocol.rs`** — `handle_timer_propose_block`

No structural change is needed here because `ctx.pending_txs` is already consumed
in arrival order. After C3, `ctx.pending_txs` is sorted descending by gas price so
the proposer automatically picks the highest-value transactions first. The only tighten:
add a `take(MAX_TXS_PER_BLOCK)` bound (new constant, see C6).

### Validation

**`etheram-node/src/implementations/ibft/ibft_protocol/ibft_protocol_validation.rs`**
- Validate incoming `PrePrepare` blocks: transactions must be sorted descending by
  `gas_price` (same rule all honest proposers follow). Reject blocks that violate
  ordering — this prevents a malicious proposer from front-running.

**`etheram-node/src/implementations/ibft/ibft_protocol/ibft_protocol_dispatch.rs`**
- `handle_client_message` — `SubmitTransaction` rejection: add `gas_price == 0`
  check (a zero-gas-price tx is rejected with a new `TransactionRejectionReason::ZeroGasPrice`).

### Rejection reason

**`etheram-node/src/executor/outgoing/external_interface/transaction_rejection_reason.rs`**
- Add `ZeroGasPrice` variant.

### Tests

**`etheram-node/tests/implementations/in_memory_cache_tests.rs`** (add cases)
- `add_pending_orders_by_gas_price_descending`
- `add_pending_per_sender_deduplication_replaces_on_higher_gas_price`
- `add_pending_per_sender_deduplication_rejects_equal_gas_price`
- `add_pending_pool_capacity_evicts_lowest_gas_price`
- `get_pending_returns_descending_order`

**`etheram-validation/tests/`** (new cluster-level test file `tx_pool_tests.rs`)
- `submit_two_txs_proposer_orders_higher_gas_price_first` — two txs from different
  senders, different gas prices; committed block must have them in descending order.
- `submit_zero_gas_price_tx_is_rejected`

---

## C6 — Block Gas Limit Enforcement

### Goal

Add a block-level gas limit so the proposer fills blocks up to the limit (ordering
by gas price, inheriting C3), and validators reject blocks that exceed it.

### Current state

`MAX_GAS_LIMIT` enforces a per-transaction ceiling at submission time. At proposal
time `pending_txs` is taken wholesale — there is no block-level cap.

### Data model changes

**`etheram-node/src/common_types/block.rs`**
- Add `pub gas_limit: Gas` field to `Block`.
- Update `Block::new(...)` — add `gas_limit: Gas` parameter.
- Update `Block::empty(...)` — pass `gas_limit: 0`.
- Include `gas_limit` in `compute_hash()` byte mix to prevent hash collisions
  between blocks at the same height with different limits.

### New constant

**`etheram-node/src/implementations/ibft/ibft_protocol.rs`**
- Add `pub(crate) const BLOCK_GAS_LIMIT: Gas = 10_000_000;` alongside `MAX_GAS_LIMIT`.
- Add `const MAX_TXS_PER_BLOCK: usize = 256;` as a hard safety cap (prevents
  pathological proposals with thousands of zero-gas txs that individually fit
  `MAX_GAS_LIMIT`).

### Proposer changes — `handle_timer_propose_block`

**`etheram-node/src/implementations/ibft/ibft_protocol.rs`**

Replace `ctx.pending_txs.clone()` with a greedy fill loop:

```
let mut selected = Vec::new();
let mut accumulated_gas: Gas = 0;
for tx in &ctx.pending_txs {           // already sorted desc by gas_price (C3)
    if selected.len() >= MAX_TXS_PER_BLOCK { break; }
    let next_gas = accumulated_gas.saturating_add(tx.gas_limit);
    if next_gas > BLOCK_GAS_LIMIT { continue; }   // skip, try cheaper txs
    accumulated_gas = next_gas;
    selected.push(tx.clone());
}
```

Pass `selected` (not `ctx.pending_txs`) to `Block::new(...)` and include
`gas_limit: BLOCK_GAS_LIMIT`.

### Validation changes

**`etheram-node/src/implementations/ibft/ibft_protocol/ibft_protocol_validation.rs`**
- Add a `valid_block_gas(block)` helper:
  - Compute `total_gas = block.transactions.iter().map(|t| t.gas_limit).sum::<Gas>()`.
  - Reject if `total_gas > block.gas_limit`.
  - Reject if `block.gas_limit != BLOCK_GAS_LIMIT` (honesty check — an honest
    proposer always uses the canonical limit).
- Call `valid_block_gas(block)` in `valid_pre_prepare(...)`.

### Receipt changes

`TransactionReceipt` already carries `gas_used` and `cumulative_gas_used`. No change
needed there. The `StoreReceipts` mutation path is unaffected.

### Tests

**`etheram-node/tests/implementations/ibft/ibft_protocol_propose_tests.rs`** (add cases)
- `propose_block_respects_block_gas_limit` — pending pool has txs whose combined
  `gas_limit` exceeds `BLOCK_GAS_LIMIT`; proposed block must only include the
  highest-gas-price prefix that fits.
- `propose_block_skips_oversized_tx_and_fits_smaller_one` — one large tx (gas_limit
  near `BLOCK_GAS_LIMIT`) followed by a small tx; the small tx must be included
  even if the large one is skipped.

**`etheram-node/tests/implementations/ibft/ibft_protocol_validation_tests.rs`**
(new file)
- `valid_pre_prepare_rejects_block_exceeding_gas_limit`
- `valid_pre_prepare_accepts_block_at_exact_gas_limit`

**`etheram-validation/tests/`** — extend `tx_pool_tests.rs`
- `block_gas_limit_caps_transactions_in_committed_block`

---

## C1 — Expand TinyEVM

### Goal

Add `MSTORE`/`MLOAD` (memory model), `CALLDATALOAD`/`CALLDATASIZE`, `SHA3`,
`JUMP`/`JUMPI` to `TinyEvmEngine`. These opcodes are required for Solidity-compiled
contract dispatch tables (ABI decoding), hash-based key addressing, and control flow.

### Scope constraint

`TinyEvmEngine` implements `ExecutionEngine` — immutable input, declarative
`ExecutionResult`, no I/O. Every new opcode is confined to `execute_bytecode`.
The `ExecutionEngine` trait signature does not change.

### New opcodes and EVM byte values

| Opcode | Byte | Gas | Istanbul schedule |
|---|---|---|---|
| `MSTORE` | `0x52` | 3 + memory expansion | Istanbul §Yellow-Paper |
| `MLOAD` | `0x51` | 3 + memory expansion | Istanbul |
| `CALLDATASIZE` | `0x36` | 2 | Istanbul |
| `CALLDATALOAD` | `0x35` | 3 | Istanbul |
| `SHA3` (KECCAK256) | `0x20` | 30 + 6 per word | Istanbul |
| `JUMP` | `0x56` | 8 | Istanbul |
| `JUMPI` | `0x57` | 10 | Istanbul |
| `JUMPDEST` | `0x5b` | 1 | Istanbul — mandatory target marker |
| `PUSH2`–`PUSH32` | `0x61`–`0x7f` | 3 each | Istanbul |
| `DUP1`–`DUP16` | `0x80`–`0x8f` | 3 each | Istanbul |
| `SWAP1`–`SWAP16` | `0x90`–`0x9f` | 3 each | Istanbul |
| `POP` | `0x50` | 2 | Istanbul |
| `SUB` | `0x03` | 3 | Istanbul |
| `MUL` | `0x02` | 5 | Istanbul |
| `DIV` | `0x04` | 5 | Istanbul |
| `EQ` | `0x14` | 3 | Istanbul |
| `LT` | `0x10` | 3 | Istanbul |
| `GT` | `0x11` | 3 | Istanbul |
| `ISZERO` | `0x15` | 3 | Istanbul |
| `AND` | `0x16` | 3 | Istanbul |
| `OR` | `0x17` | 3 | Istanbul |
| `CALLER` | `0x33` | 2 | Istanbul |
| `CALLVALUE` | `0x34` | 2 | Istanbul |
| `REVERT` | `0xfd` | 0 | Istanbul — maps to OutOfGas (no return data model yet) |

`JUMP`/`JUMPI` require JUMPDEST validation (pre-scan the bytecode for valid
jump destinations before execution begins).

### Memory model

The EVM memory is a byte-addressable array indexed from 0, expanding in 32-byte
words on demand. It exists for the duration of a single call frame and is not
persisted. Gas for memory expansion uses the quadratic formula from the Yellow Paper:

```
memory_cost(words) = 3 * words + words^2 / 512
expansion_cost = memory_cost(new_words) - memory_cost(old_words)
```

Represent memory as `Vec<u8>` inside `execute_bytecode`. Track `memory_words: u64`
(current high-water mark in 32-byte words). Charge expansion when `MSTORE` or
`MLOAD` access an address beyond the current high-water mark.

### Changes: `tiny_evm_opcode.rs`

- Extend `TinyEvmOpcode` enum with one variant per new opcode listed above.
- Add `OPCODE_*` byte constants.
- Update `decode_opcode` and `opcode_name` match arms.
- PUSH2–PUSH32 can be represented as `Push(u8)` where the payload byte encodes the
  count (1–32), or as a flat enum. Use `Push(u8)` to keep arms manageable.

### Changes: `tiny_evm_gas.rs`

- Add a constant for every new fixed-cost opcode.
- `GAS_MSTORE` / `GAS_MLOAD` = `3` (base; expansion is computed separately).
- `GAS_SHA3_BASE` = `30`, `GAS_SHA3_WORD` = `6`.
- `GAS_JUMP` = `8`, `GAS_JUMPI` = `10`, `GAS_JUMPDEST` = `1`.
- Memory expansion helper function `memory_expansion_cost(old_words, new_words) -> Gas`.

### Changes: `tiny_evm_engine.rs`

The `execute_bytecode` function signature gains two new parameters passed by value:
- `calldata: &[u8]` — the transaction's `data` field (already available as
  `transaction.data`; pass `&transaction.data` at the call site).
- The memory `Vec<u8>` is a local variable inside `execute_bytecode`.

Add helper functions:
- `exec_mstore(stack, memory, gas)` — pops `offset` and `value`; writes 32 bytes
  into memory at `offset`; charges expansion; panics/returns OutOfGas if overflow.
- `exec_mload(stack, memory, gas)` — pops `offset`; reads 32 bytes from memory;
  charges expansion if necessary; pushes as `Hash`.
- `exec_calldatasize(stack, calldata)` — pushes `calldata.len() as u128`.
- `exec_calldataload(stack, calldata)` — pops `offset`; reads a 32-byte word from
  `calldata` starting at `offset` (zero-padded if beyond end); pushes result.
- `exec_sha3(stack, memory, gas)` — pops `offset` and `size`; computes keccak256
  over `memory[offset..offset+size]`; charges `GAS_SHA3_BASE + GAS_SHA3_WORD *
  ceil(size / 32)`; pushes result. Use `sha3 = "0.10"` crate (already in
  workspace dependencies if present; otherwise add it).
- `exec_jump(stack, pc)` — pops `dest`; validates `dest` is in `jumpdests` set;
  sets `pc = dest`; returns `OutOfGas` if invalid.
- `exec_jumpi(stack, pc)` — pops `dest`, `cond`; jumps iff `cond != 0`.
- `exec_push_n(stack, bytecode, pc, n)` — reads `n` bytes from `bytecode[pc..]`,
  zero-pads left to 32 bytes, pushes as `Hash`, advances `pc` by `n`.
- `exec_dup(stack, n)` — copies stack element `n` deep (1-indexed from top).
- `exec_swap(stack, n)` — swaps top with element `n+1` deep.
- `exec_pop(stack)` — discards top.
- `exec_sub/mul/div/eq/lt/gt/iszero/and/or(stack)` — trivial arithmetic on
  `word_to_u128` values.
- `exec_caller(stack, transaction)` — pushes `transaction.from` zero-padded to 32 bytes.
- `exec_callvalue(stack, transaction)` — pushes `transaction.value` as 32-byte word.
- `exec_revert(...)` — returns `(TransactionStatus::OutOfGas, 0, Vec::new())`.

JUMPDEST pre-scan: before the main loop, collect the set of valid jump destinations:
```rust
let jumpdests: BTreeSet<usize> = precompute_jumpdests(bytecode);
```
`precompute_jumpdests` walks the bytecode, skips PUSH immediates, and records
positions of `JUMPDEST` bytes.

### Keccak dependency

Add `tiny-keccak = { version = "2", features = ["keccak"] }` to workspace
`Cargo.toml` under `[workspace.dependencies]` and reference it in `etheram-node`'s
`[dependencies]` via `.workspace = true`. Verify it is `no_std`-compatible
(it is — the crate has `#![no_std]`).

### Tests

**`etheram-node/tests/implementations/tiny_evm_engine_tests.rs`** (new file)

One test per new opcode group, following the AAA pattern:
- `mstore_mload_roundtrip`
- `calldatasize_returns_data_length`
- `calldataload_reads_padded_word`
- `sha3_produces_correct_keccak256` — use a known vector (e.g. keccak256 of empty
  string = `0xc5d246...`).
- `jump_to_jumpdest_succeeds`
- `jumpi_not_taken_when_condition_zero`
- `jumpi_taken_when_condition_nonzero`
- `jump_to_non_jumpdest_returns_out_of_gas`
- `push2_encodes_two_byte_immediate`
- `dup1_duplicates_top`
- `swap1_exchanges_top_two`
- `caller_pushes_sender_address`
- `callvalue_pushes_transfer_amount`
- `memory_expansion_gas_charged`
- `revert_returns_out_of_gas_status`

**Proptest** (`etheram-node/tests/implementations/ibft/prop_tests/peer_message_proptest_tests.rs`)
- Add a property: for arbitrary valid bytecode sequences (STOP / PUSH1 / ADD / RETURN),
  `execute_bytecode` never panics and always returns either `Success` or `OutOfGas`.

---

## Cross-Cutting Obligations (all three features)

### No warnings, `cargo fmt`, `no_std` gate

After every change:
1. `cargo fmt` from workspace root.
2. `cargo check -p etheram-node --no-default-features` must pass.
3. `powershell -File scripts\run_tests.ps1` must pass.
4. For productive code changes: `powershell -File scripts\run_apps.ps1` must pass.

### Observer trait

Every new `Action` variant or new `StorageMutation` kind must be reflected in
the `ActionKind` enum in `etheram-node/src/observer.rs` and handled in
`SemihostingObserver` in `etheram-embassy/src/semihosting_observer.rs`.

### Dependency direction

All changes are confined to `etheram-node/` and `etheram-validation/`.
`etheram-embassy/` is touched only for `SemihostingObserver` observer updates
and Stage 3 scenario exercises (after Stage 1 + 2 are green).

### Dual-layer test mandate

Every productive code change requires tests at both layers before the feature is
considered complete:
- **Stage 1**: protocol-level unit tests in `etheram-node/tests/`.
- **Stage 2**: cluster-level tests in `etheram-validation/tests/`.

### etheram-embassy Stage 3 exercise

After Stage 1 + 2 are green for each feature, update `etheram-embassy/src/main.rs`
to exercise the feature minimally in QEMU:
- **C3**: submit two txs with different gas prices; assert higher-gas-price tx
  appears first in the committed block.
- **C6**: submit enough txs to exceed `BLOCK_GAS_LIMIT`; assert that only the
  highest-value subset is committed and the rest remain pending.
- **C1**: submit a tx with non-empty `data` containing a short bytecode program
  that uses `MSTORE`/`MLOAD` and verify it commits successfully (no `OutOfGas`).
