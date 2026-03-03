# Block Re-Execution Validation Plan

**Goal:** Non-proposer validators re-execute received blocks independently (via `ExecutionEngine`) and compare the resulting state root and receipts against the proposer's claims before voting `Prepare`. This closes the trust gap where validators currently accept proposed blocks based on surface-level checks only.

**Why this matters:**
- This is how real Ethereum validators work — they don't trust the proposer
- It's the single most architecturally significant feature remaining: it wires the execution engine into the _consensus path_ (not just the post-commit path), validating that the 3-6 model holds when execution and consensus intersect
- It transforms `valid_block()` from a lightweight nonce/balance/gas sanity check into a full deterministic state transition verification

**Prerequisite:** All existing tests green (`scripts\test.ps1` exit 0).

---

## Current State (what happens today)

### Proposer (correct)
1. Timer fires `ProposeBlock` → `handle_timer_propose_block()` builds a `Block` from pending transactions
2. Block is broadcast as `PrePrepare`
3. On commit, `handle_commit()` emits `ExecuteBlock { block }` → the node's step loop calls `execution_engine.execute()` → applies successful mutations, reverts out-of-gas transactions, stores receipts

### Non-proposer validator (incomplete)
1. Receives `PrePrepare` → calls `valid_pre_prepare()` → calls `valid_block()`
2. `valid_block()` checks: state root match, nonce validity, balance sufficiency, gas limit bounds — **but does not execute the block**
3. If valid, immediately votes `Prepare`
4. On commit, the same `ExecuteBlock` flow runs — but by this point the node has already voted

### The gap
Validators vote `Prepare` without knowing whether the block's transactions will _actually_ succeed under the execution engine. A proposer could include a contract-calling transaction that runs out of gas, and validators would still vote for it because `valid_block()` only checks value-transfer preconditions.

---

## Design Decisions

### D1: Where does re-execution happen?

**Option A (chosen):** Inside `IbftProtocol::valid_block()` — the protocol calls a re-execution function as part of block validation, before voting Prepare.

**Why not Option B (re-execute in the step loop)?** This would require the step loop to intercept `PrePrepare` processing and inject results back into the protocol, breaking the pure `handle_message → actions` contract. The protocol must remain the sole decision-maker.

**Why not Option C (ContextBuilder supplies pre-computed results)?** Over-broadens the context. Re-execution is block-specific, not message-specific. It would also require the ContextBuilder to speculatively execute every possible block, which is impossible.

### D2: How does the protocol access the ExecutionEngine?

The protocol is pure — no I/O, no side effects. But `ExecutionEngine::execute()` is a pure computation (immutable inputs → declarative result). It qualifies as a dependency the protocol can hold, similar to `SignatureScheme`.

**Approach:** `IbftProtocol` receives a `BoxedExecutionEngine` at construction time. `valid_block()` calls it to re-execute the proposed block and compare the resulting state against expectations.

### D3: What does re-execution verify?

1. **Post-execution state root** — the re-executor computes what the state root _would_ be after applying the block. The proposer must include this expected post-state root in the block (new `Block::post_state_root` field). If they don't match, the block is invalid.
2. **Receipt consistency** — the re-executor produces `TransactionResult`s. For each transaction, the status (Success/OutOfGas) and `gas_used` must be deterministic. If the proposer claims a transaction succeeded but the re-executor says OutOfGas (or vice versa), the block is invalid.

### D4: What new data does the Block carry?

Currently `Block` carries: `height`, `proposer`, `transactions`, `state_root` (pre-state). To support re-execution validation, the proposer must commit to the _outcome_:

- `post_state_root: Hash` — the state root after applying all transactions
- `receipts_root: Hash` — a deterministic hash of the transaction receipts (status, gas_used)

Validators re-execute, compute their own `post_state_root` and `receipts_root`, and reject the block if they disagree.

### D5: What about contract storage in the Context?

Currently `Context` carries `accounts: BTreeMap<Address, Account>` but not `contract_storage`. The `ExecutionEngine` needs both. Two sub-decisions:

- **Phase 1:** Add `contract_storage: BTreeMap<(Address, Hash), Hash>` to `Context`. `EagerContextBuilder` already has access to `EtheramState` which exposes `snapshot_contract_storage()`.
- **Phase 2:** The protocol's `valid_block()` uses both `ctx.accounts` and `ctx.contract_storage` when calling the execution engine.

---

## Phase 1 — Block Outcome Commitment (etheram)

Extend the `Block` type to carry proposer commitments to execution results, and define the receipts root computation.

### 1.1 Add `post_state_root` and `receipts_root` to `Block`

File: `etheram/src/common_types/block.rs`

```rust
pub struct Block {
    pub height: Height,
    pub proposer: PeerId,
    pub transactions: Vec<Transaction>,
    pub state_root: Hash,         // pre-execution state root (existing)
    pub post_state_root: Hash,    // state root after executing all transactions (new)
    pub receipts_root: Hash,      // deterministic hash of transaction receipts (new)
}
```

Both fields default to `[0u8; 32]` in `Block::new()` for backward compatibility during phased rollout.

### 1.2 Define `compute_receipts_root`

File: `etheram/src/execution/receipts_root.rs`

Deterministic hash over the ordered list of `(TransactionStatus, gas_used)` tuples — XOR-mix or similar to match `compute_state_root` style. This function is pure and `no_std`-compatible.

### 1.3 Define `compute_post_state_root`

Reuse the existing `compute_state_root` logic (XOR-mix over sorted account map). The function already exists in `etheram/src/common_types/state_root.rs` — it just needs to be called with the _post-execution_ account map.

### 1.4 Update `Block::compute_hash`

Include `post_state_root` and `receipts_root` in the block hash computation so that validators vote on the full commitment.

### 1.5 Validation — Stage 1

- Existing tests continue to pass (new fields are zero-initialized where not set)
- New unit tests for `compute_receipts_root` determinism and ordering
- New unit tests for `Block::compute_hash` including new fields

---

## Phase 2 — Context Extension (etheram + etheram-variants)

Make contract storage available to the protocol for re-execution.

### 2.1 Add `contract_storage` to `Context`

File: `etheram/src/context/context_dto.rs`

```rust
pub struct Context {
    pub peer_id: PeerId,
    pub current_height: Height,
    pub state_root: Hash,
    pub accounts: BTreeMap<Address, Account>,
    pub contract_storage: BTreeMap<(Address, Hash), Hash>,  // new
    pub pending_txs: Vec<Transaction>,
}
```

### 2.2 Update `EagerContextBuilder`

File: `etheram-variants/src/implementations/eager_context_builder.rs`

Populate `contract_storage` from `state.snapshot_contract_storage()`. Initially, include the full snapshot — optimization (loading only relevant slots) is a future concern.

### 2.3 Update all `Context::new()` call sites

Every test or builder that constructs a `Context` must include the new field. Most will pass an empty `BTreeMap::new()`.

### 2.4 Validation — Stage 1

- All existing tests compile and pass with the extended `Context`
- No behavioral changes yet

---

## Phase 3 — Proposer Block Construction with Commitments (etheram-variants)

The proposer pre-executes the block to compute `post_state_root` and `receipts_root` before broadcasting.

### 3.1 Give `IbftProtocol` an `ExecutionEngine`

Add `execution_engine: BoxedExecutionEngine` to `IbftProtocol` fields. Injected at construction time via the builder.

### 3.2 Update `handle_timer_propose_block`

Before broadcasting `PrePrepare`, the proposer:

1. Builds the block (existing logic)
2. Calls `self.execution_engine.execute(&block, &ctx.accounts, &ctx.contract_storage)`
3. Computes `post_state_root` from the resulting post-execution account map
4. Computes `receipts_root` from the resulting `TransactionResult`s
5. Sets `block.post_state_root` and `block.receipts_root`
6. Broadcasts the enriched block

### 3.3 Helper: `execute_and_compute_commitments`

A pure method on `IbftProtocol` that takes `(&Block, &BTreeMap<Address, Account>, &BTreeMap<(Address, Hash), Hash>)` and returns `(Hash, Hash)` — the `post_state_root` and `receipts_root`. Used by both the proposer (Phase 3) and the validator (Phase 4).

The post-execution account map is computed by applying successful `TransactionResult` mutations to a working copy of the accounts. Out-of-gas transactions produce no mutations. This mirrors the step loop's logic, but the protocol computes the _expected_ outcome rather than actually mutating state.

### 3.4 Update `ProtocolBuilder` / `IbftProtocolBuilder`

File: `etheram-variants/src/builders/protocol_builder.rs`

Add `with_execution_engine(engine)` to the builder. Require it for IBFT variant construction.

### 3.5 Validation — Stage 1

- Proposer tests: verify that `PrePrepare` blocks carry correct `post_state_root` and `receipts_root`
- Round-trip: execute block manually, confirm roots match

---

## Phase 4 — Validator Re-Execution (etheram-variants)

Non-proposer validators re-execute the proposed block and reject it if outcomes disagree.

### 4.1 Extend `valid_block()` with re-execution

File: `etheram-variants/src/implementations/ibft/ibft_protocol/ibft_protocol_validation.rs`

After the existing nonce/balance/gas checks, add:

```
1. Call self.execute_and_compute_commitments(block, &ctx.accounts, &ctx.contract_storage)
2. If computed post_state_root != block.post_state_root → return false
3. If computed receipts_root != block.receipts_root → return false
4. Return true
```

The existing surface-level checks remain as fast-reject guards (they're cheaper than execution). Re-execution is the final validation gate.

### 4.2 Stage 1 Protocol Tests

New test file: `etheram-variants/tests/implementations/ibft/ibft_protocol_reexecution_tests.rs`

| Test | Scenario |
|---|---|
| `valid_block_with_correct_commitments_accepted` | Proposer computes correct roots → validator accepts |
| `valid_block_with_wrong_post_state_root_rejected` | Tamper with `post_state_root` → validator rejects |
| `valid_block_with_wrong_receipts_root_rejected` | Tamper with `receipts_root` → validator rejects |
| `valid_block_with_out_of_gas_transaction_correct_roots` | Block contains a tx that runs out of gas; proposer correctly reflects OOG in roots → accepted |
| `valid_block_with_out_of_gas_transaction_wrong_roots` | Proposer claims OOG tx succeeded → validator re-executes, disagrees, rejects |
| `valid_block_with_contract_storage_mutations_accepted` | Block with SSTORE transaction → re-execution produces same contract storage root → accepted |
| `valid_block_empty_transactions_accepted` | Empty block → trivially correct roots → accepted |
| `reexecution_with_locked_block_accepted` | Re-proposed locked block (via PreparedCertificate) carries correct roots → accepted |

### 4.3 Stage 1 Existing Test Updates

All tests that construct blocks for `PrePrepare` messages must now include valid `post_state_root` and `receipts_root`. Introduce a test helper: `build_block_with_commitments(block, accounts, contract_storage, engine)` that pre-computes the roots.

---

## Phase 5 — Cluster Validation (etheram-validation)

### 5.1 Cluster re-execution tests

New test file: `etheram-validation/tests/ibft_cluster_reexecution_tests.rs`

| Test | Scenario |
|---|---|
| `cluster_honest_block_commits_with_reexecution` | 4 honest nodes — block with transactions commits successfully; all validators independently verified execution |
| `cluster_proposer_with_wrong_post_state_root_rejected` | Inject a Byzantine proposer that lies about `post_state_root` → block not committed, round times out |
| `cluster_proposer_with_wrong_receipts_root_rejected` | Byzantine proposer lies about `receipts_root` → block not committed |
| `cluster_mixed_block_with_gas_failure_commits` | Block with success + OOG transactions → all nodes agree on partial execution |
| `cluster_contract_execution_block_commits` | Block with SSTORE bytecode tx → re-execution at all nodes produces matching contract storage root |
| `cluster_view_change_after_invalid_block` | Byzantine proposer sends invalid block → round timeout → view change → next proposer succeeds |

### 5.2 Register test module

Add `pub mod ibft_cluster_reexecution_tests;` to `etheram-validation/tests/all_tests.rs`.

### 5.3 Update existing cluster tests

Existing tests that build or inject blocks must include valid execution commitments. Update cluster test helpers to automatically compute `post_state_root` and `receipts_root` when constructing test blocks.

---

## Phase 6 — Embassy Stage 3 (etheram-embassy)

### 6.1 Wire `ExecutionEngine` into `IbftProtocol` in Embassy setup

Both configurations (`in-memory` and `real`) already construct an `IbftProtocol` and an `ExecutionEngine`. Connect them so the protocol holds a reference to the engine.

### 6.2 Add Act: Re-execution demonstration

After the existing Acts, add a new Act that demonstrates re-execution:

1. Submit a contract-calling transaction (SSTORE bytecode)
2. Wait for block commitment
3. Log that all 5 nodes independently re-executed and agreed on the post-state root
4. (Optional) Submit a second transaction with known OOG outcome and verify the block still commits with correct partial-failure roots

### 6.3 Verify both configurations

- `run_channel_in_memory.ps1` — TinyEvmEngine re-execution via in-memory transport
- `run_udp_semihosting.ps1` — TinyEvmEngine re-execution via UDP + semihosting

### 6.4 Observer updates

Ensure `SemihostingObserver` logs re-execution outcomes (or add a new `ActionKind::BlockReexecuted` variant if appropriate for observability).

---

## Phase 7 — Cleanup and Hardening

### 7.1 Update `ROADMAP.md`

Mark item 4 (Minimal EVM Execution) status update — re-execution validation closes the "architectural proof (protocol purity under computation)" gap.

### 7.2 Update `IBFT-ROADMAP.md`

Add "Block re-execution validation" to Implemented Features.

### 7.3 Remove zero-default for `post_state_root` / `receipts_root`

Once all call sites are migrated, remove the backward-compatibility defaults. All block construction must explicitly set execution commitments.

### 7.4 Evaluate Context efficiency

With full contract storage snapshots in Context, measure memory impact. If excessive for large state, consider adding a `StorageQuery::GetContractStorage(address, slot)` lazy-loading path. This is an optimization — correctness first.

---

## Phasing Summary

| Phase | Scope | Crates | Risk | Effort |
|---|---|---|---|---|
| 1 | Block outcome commitment fields | `etheram` | Medium — touches Block type used everywhere | Medium |
| 2 | Context extension with contract storage | `etheram`, `etheram-variants` | Low — additive field | Low |
| 3 | Proposer pre-execution | `etheram-variants` | Medium — changes block construction | Medium |
| 4 | Validator re-execution | `etheram-variants` | Low — extends existing validation | Medium |
| 5 | Cluster tests | `etheram-validation` | Low — additive tests | Medium |
| 6 | Embassy integration | `etheram-embassy` | Low — wiring + new Act | Low |
| 7 | Cleanup | all | Low | Low |

**Phase 1 is the hard part.** Adding fields to `Block` propagates to every test, builder, and serialization path. Get Phase 1 green first — Phases 2-7 are incremental on top.

**Total estimated scope:** ~800–1200 lines of productive code, ~1500–2000 lines of test code.

---

## Out of Scope (Future Work)

These build on re-execution validation but are separate increments:

- **Parallel re-execution** — execute transactions concurrently where dependency analysis permits
- **Optimistic execution** — begin re-execution speculatively during PrePrepare before voting, abort if invalid
- **Fraud proofs** — instead of full re-execution, verify compact proofs of invalid execution
- **Block gas limit** — cap total gas per block during proposal construction (complementary to re-execution)
- **Merkle Patricia Trie state root** — replace XOR-mix with a real MPT for verifiable inclusion proofs
- **State witness** — proposer includes only the accessed state slots, enabling stateless validation
