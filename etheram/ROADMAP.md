# EtheRAM – Feature Set & Implementation Roadmap

**Status:** Active — Phase 1 complete, Phase 2 partial, Phase 3–4 largely complete, Phase 5 partial
**Purpose:** Define *what* EtheRAM must implement to be considered meaningful, and *in which order* those features should be built to maximize learning, correctness, and validation of the 3‑6 architecture.

This document is intentionally opinionated. It reflects architectural goals, not ecosystem completeness.

**Related:** [Istanbul BFT Implementation Plan](IBFT-ROADMAP.md) – Detailed consensus implementation strategy

---

## Guiding Principles

- **Architecture > completeness**: Each feature must stress-test the 3‑6 model.
- **Minimal but real Ethereum**: No toy abstractions; simplified, but semantically honest.
- **Executable specification**: Features should be verifiable through behavior, not claims.
- **Replaceability**: Storage, execution, transport, and coordination must be swappable as wholes.

---

## MUST HAVE (Project-Critical)

These define whether EtheRAM is *real*.
If any of these are missing, the project does not substantiate its claims.

### 1. Canonical Block & State Model ✅

**What:**
- Block header (parent hash, state root, tx root, number)
- Block body (transactions)
- Canonical chain selection (longest / heaviest rule)

**Why:**
- Grounds the protocol in an actual blockchain model
- Enables state transitions, reorgs, and verification

**Status:** Block, Account, and Hash types implemented. Deterministic `compute_state_root` via XOR-mix over sorted `BTreeMap<Address, Account>`. Note: canonical chain selection / reorg is not implemented (single committed chain only).

---

### 2. In-Memory Ethereum-like State ✅

**What:**
- Accounts with nonce, balance, code hash, storage root
- Deterministic state transitions
- State root recomputation

**Why:**
- Core correctness surface of Ethereum
- Stress-tests Context construction and action execution

**Status:** `InMemoryStorage` and `InMemoryCache` implemented. `UpdateAccount` mutations applied on commit. State root auto-recomputed on every mutation. `EagerContextBuilder` supplies account map and state root to protocol.

---

### 3. Transaction Lifecycle ✅

**What:**
- Signed transactions (simplified cryptography acceptable)
- Nonce validation
- Balance checks
- Inclusion in blocks

**Why:**
- Forces interaction between client, protocol, and state
- Validates unified message handling

**Status:** Transactions submitted via `ClientRequest`; nonce validation, balance checks, and gas-limit checks run before block inclusion. `InsufficientBalance`, `InvalidNonce`, and `GasLimitExceeded` rejections exercised in tests and QEMU Acts 3/5/6.

---

### 4. Minimal EVM Execution ✅

**What:**
- Subset of EVM opcodes (ADD, MUL, SSTORE, SLOAD, CALL, RETURN)
- Gas accounting (simplified but enforced)
- Deterministic execution

**Why:**
- This is where most blockchains collapse architecturally
- Proves protocol purity under heavy computation

**Status:** `TinyEvmEngine` opcode execution and gas accounting are implemented, including `SSTORE` behavior and deterministic receipts. IBFT proposer-side commitment construction and validator re-execution validation are integrated, validating protocol purity under computation.

---

### 5. Unified Protocol Entry Point ✅

**What:**
- Single `handle_message(source, message, context)`
- Peer, client, and timer messages treated uniformly

**Why:**
- Core claim of the 3‑6 model
- Eliminates artificial consensus separation

**Status:** `IbftProtocol::handle_message` dispatches on `MessageSource` (Peer / Client / Timer) through a single entry point. `TypeBasedPartitioner` cleanly separates mutations from outputs.

---

### 6. Swappable Data Layer (Storage + Cache) ✅

**What:**
- Replace entire state backend without touching protocol
- At least two implementations:
  - Pure in-memory
  - Alternative (e.g., copy-on-write, transactional, or snapshot-based)

**Why:**
- Demonstrates abstraction correctness
- Enables deep testing and mocking

**Status:** `InMemoryStorage`, `InMemoryCache` (development), `SemihostingStorage` (embedded). Both swap without touching protocol logic. `StorageVariant` and `StorageBuilder` in `etheram-variants` provide the builder-pattern swap path.

---

### 7. Deterministic Single-Node Execution ✅

**What:**
- Single-node chain execution
- Fully deterministic replay from genesis

**Why:**
- Baseline for correctness
- Enables exhaustive testing and fuzzing

**Status:** `EtheramNode::step` is fully deterministic. Genesis accounts seeded via `NodeInfraSlot::with_genesis_account`. WAL enables deterministic restart recovery. 12 unit tests in `etheram/tests/`.

---

## SHOULD HAVE (Strong Validation)

These elevate EtheRAM from *correct* to *important*.

### 8. Multi-Node Logical Network (In-Memory) ✅

**What:**
- Multiple nodes as tasks or threads
- In-memory transport abstraction
- Message passing via protocol actions

**Why:**
- Validates coordination layer neutrality
- Enables fork and reconciliation scenarios

**Status:** `IbftCluster` / `IbftTestNode` harness in `etheram-validation`. 115 cluster and Byzantine tests. Five-node Embassy async cluster with `InMemoryTransport` and UDP-over-MockNetDriver.

---

### 9. Forks & Reorgs ❌

**What:**
- Competing blocks
- Chain reorganization
- State rollback / recomputation

**Why:**
- Forces clean separation of protocol vs state
- Tests action reversibility and determinism

**Status:** Not implemented. IBFT's safety guarantee prevents forks under honest-majority conditions, so this path is never exercised. State rollback and reorg logic do not exist.

---

### 10. Timer-Driven Block Production ✅

**What:**
- Block proposal via timer events
- No background threads in protocol

**Why:**
- Validates timer as first-class message source
- Confirms unified message handling

**Status:** `TimerEvent::ProposeBlock` drives block proposals. `InMemoryTimer` and Embassy timer channel are the two implementations. No background threads; the protocol is stepped only in response to timer events injected through `IncomingSources`.

---

### 11. Protocol-Level Property Testing ❌

**What:**
- Property tests on `handle_message`
- Invariants (no double-spend, monotonic height, gas safety)

**Why:**
- Demonstrates protocol purity
- Supports formal reasoning claims

**Status:** Not implemented. Extensive example-based tests exist across 325+ test cases in `etheram-variants` and 115 in `etheram-validation`, but no randomized property tests (e.g., proptest / quickcheck) have been written.

---

### 12. Action Inspection & Logging ✅

**What:**
- Actions treated as inspectable data
- Ability to log, diff, and replay action sets

**Why:**
- Enables debugging and correctness auditing
- Supports alternative execution strategies

**Status:** `Observer` trait with `ActionKind` enum (non-generic projection of `Action<M>`) allows per-action callbacks for context, mutations, and outputs. `SemihostingObserver` logs per-item detail in QEMU. Test harnesses assert on emitted `ActionKind`s.

---

## NICE TO HAVE (Stretch / Research)

These are not required for relevance but expand impact.

### 13. Transactional / Atomic State Execution ❌

**What:**
- Batch action execution with commit/rollback

**Why:**
- Bridges toward production semantics
- Supports reorg safety

**Status:** Not implemented. Mutations are applied sequentially with no rollback capability.

---

### 14. Parallel Action Execution ❌

**What:**
- Independent actions executed concurrently

**Why:**
- Performance exploration
- Validates action decomposition

**Status:** Not implemented.

---

### 15. No-Std + Embedded Target ✅

**What:**
- Core protocol usable in no-std environments

**Why:**
- Demonstrates extreme portability
- Reinforces protocol purity

**Status:** `core`, `etheram`, and `etheram-variants` carry `#![no_std]` and use `alloc` throughout. `etheram-embassy` runs the full IBFT cluster on ARM Cortex-M4 under Embassy with no standard library. Verified by `cargo check -p barechain-etheram-variants --no-default-features` in the CI gate.

---

### 16. External Execution Engines ❌

**What:**
- Swap EVM with alternative VM

**Why:**
- Confirms protocol/execution decoupling

**Status:** Not implemented. No EVM exists to swap out.

---

## NOT REQUIRED / LATER STAGE

Explicitly *out of scope* for initial success.

- ~~Networking over real TCP/UDP~~ — **done**: `embassy-net` UDP sockets over `MockNetDriver` in the real configuration
- ~~Full Ethereum cryptography~~ — **done** (scoped): Ed25519 via `ed25519-dalek` integrated into `IbftProtocol`; `PreparedCertificate` carries cryptographic quorum proof
- Full EVM opcode set
- JSON-RPC compatibility
- Peer discovery
- Proof-of-Work / Proof-of-Stake economics
- Performance optimization

---

## Recommended Implementation Order

This order minimizes rework and maximizes architectural validation.

### Phase 1 – Deterministic Core ✅ Complete
1. Canonical block & state model ✅
2. In-memory Ethereum-like state ✅
3. Transaction lifecycle ✅
4. Unified protocol entry point ✅
5. Single-node deterministic execution ✅

**Outcome:** Executable Ethereum-like state machine

---

### Phase 2 – Execution Semantics ✅ Complete
6. Minimal EVM execution ✅
7. Gas accounting ✅
8. Action modeling and execution ✅

**Outcome:** Complete — computation and gas are validated end-to-end, including proposer commitments and validator re-execution.

---

### Phase 3 – Architecture Stress Test ⚠️ Partial
9. Swappable data layer ✅
10. Forks & reorgs ❌
11. Timer-driven block production ✅

**Outcome:** The 3-6 model is validated for swappability and timer-driven coordination. Reorg path untested.

---

### Phase 4 – Distributed Semantics ✅ Largely complete
12. Multi-node logical network ✅
13. Protocol-level property testing ❌
14. Action inspection & replay ✅

**Outcome:** Multi-node IBFT consensus with Byzantine fault injection and full action observability. Formal property testing is the remaining gap.

---

### Phase 5 – Research Extensions ⚠️ Partial
15. Transactional execution ❌
16. Parallel execution ❌
17. no-std target ✅

---

## Success Criteria

EtheRAM is **relevant and important** if:
- All MUSTs and SHOULDs are implemented
- Protocol remains pure and stateless
- Data layer is fully swappable
- Multi-node behavior emerges without architectural hacks
- The node remains an ultra-thin coordinator

At that point, EtheRAM is no longer a project — it is a **position**.

---

## Current Gap Summary

| Item | Status |
|---|---|
| 1–3, 5–8, 10, 12 | ✅ Done |
| 4 — EVM opcode execution | ⚠️ Gas enforcement done; no opcode engine |
| 9 — Forks & reorgs | ❌ Not started |
| 11 — Property testing | ❌ Not started |
| 13 — Transactional execution | ❌ Not started |
| 14 — Parallel execution | ❌ Not started |
| 15 — no-std target | ✅ Done |
| 16 — External VMs | ❌ Not started (no EVM to swap) |

