# EtheRAM — Copilot Instructions

## Purpose

EtheRAM is a **research framework** for blockchain node decomposition and abstraction. It prioritizes architecture, testability, and swappability over feature completeness. The primary artefact is **EtheRAM**: a minimal but real Ethereum-like node validating the 3-6 architectural model under Byzantine consensus, embedded constraints, and Ethereum semantics. A second protocol family (**Raft**) is being implemented to prove the decomposition generalizes across consensus families.

---

## Workspace Structure

```
core/                       # Shared abstractions (PeerId, base traits)
etheram-node/               # Core Ethereum-like node (std/no_std-compatible crate)
etheram-validation/         # Cluster/integration tests (multi-node)
etheram-embassy/            # no-std + Embassy embedded port
raft-node/                  # Raft node — 3-6 model (no_std, #![no_std])
raft-validation/            # Raft cluster tests
raft-embassy/               # Raft on Embassy
```

**The etheram ecosystem is designed to become a standalone repo.** Keep `etheram*` crates self-contained. They depend on `core` only.

**The raft ecosystem mirrors the etheram ecosystem.** Keep `raft-*` crates self-contained. They depend only on `core/` — zero cross-dependencies between the two protocol families.

---

## The 3-6 Architectural Model

Every node is decomposed across:

### 6 Dimensions (structural — what components exist)
1. **Protocol** — consensus logic (pure, stateless)
2. **Storage** — persistent state
3. **Cache** — volatile working state
4. **Transport** — peer-to-peer communication
5. **ExternalInterface** — client request/response
6. **Timer** — time-based event scheduling

### 3 Spaces (functional — what roles components play)
1. **Brain Space** — build context, handle message, produce actions
2. **Scheduler Space** — poll dimensions, select next event, dispatch
3. **Dimension Space** — data dimensions (Storage, Cache) and I/O dimensions (Transport, ExternalInterface, Timer)

---

## EtheRAM Node Architecture

### `etheram-node/` — Single Node Logic

```rust
pub struct EtheramNode {
    // Infrastructure (dimensions)
    peer_id: PeerId,
    incoming: IncomingSources,       // polls Timer, ExternalInterface, Transport
    state: EtheramState,             // wraps Storage + Cache
    executor: EtheramExecutor,       // executes output actions
    // Decision (all swappable via trait objects)
    context_builder: Box<dyn ContextBuilder>,
    brain: BoxedProtocol,
    partitioner: Box<dyn Partitioner>,
}
```

**The step loop:**
```rust
fn step(&mut self) -> bool {
    if let Some((source, message)) = self.incoming.poll() {
        let context = self.context_builder.build(&self.state, self.peer_id, &source, &message);
        let actions = self.brain.handle_message(&source, &message, &context);
        let (mutations, outputs) = self.partitioner.partition(&actions);
        self.state.apply_mutations(&mutations);
        self.executor.execute_outputs(&outputs);
        return true;
    }
    false
}
```

Concrete implementations, `*Variant` enums, and `*Builder` APIs live directly in `etheram-node/`.

### `etheram-validation/` — Multi-Node Cluster Testing

- Orchestrates multiple `EtheramNode` instances
- Contains `EtheramCluster` builder
- Hosts integration/cluster tests in `tests/`
- Stage 2 validation: distributed correctness

### `etheram-embassy/` — Embedded Port Structure

The embassy crate organizes all hardware-variant code under two top-level areas under `src/`:

- **`src/infra/`** — the three independently feature-gated implementation axes
- **`src/configurations/`** — the two wiring configurations that combine the axes

```
src/
  infra/
    external_interface/
      channel/          ← in-memory channel (channel-external-interface feature)
        channel_external_interface.rs
        client_request_hub.rs
      udp/              ← UDP-backed (udp-external-interface feature)
        udp_external_interface.rs
    storage/
      in_memory/        ← in-memory (in-memory-storage feature)
        in_memory_storage.rs
      semihosting/      ← semihosting file I/O (semihosting-storage feature)
        semihosting_storage.rs
    transport/
      channel/          ← Embassy channels (channel-transport feature)
        channel_transport_hub.rs
        outbox_transport.rs
      udp/              ← UDP serialized (udp-transport feature)
        udp_transport.rs
        wire_ibft_message.rs
  configurations/
    in_memory/
      setup.rs          ← wires channel transport + in-memory storage + channel EI
    real/
      setup.rs          ← wires UDP transport + semihosting storage + UDP EI
```

Each axis (`transport`, `storage`, `external_interface`) is independently feature-gated. Adding a new variant means adding a subfolder under the relevant `src/infra/` axis and updating only that axis's `mod.rs`. Each axis has mutual-exclusivity `compile_error!` guards in `configurations/mod.rs`.

### `etheram-embassy/` — Two Mandatory Configurations

The embassy project must always maintain exactly two working configurations end-to-end:

| Configuration | Transport | Storage | External Interface | Script |
|---|---|---|---|---|
| **all-in-memory** | `channel-transport` | `in-memory-storage` | `channel-external-interface` | `run_channel_in_memory.ps1` |
| **real** | `udp-transport` | `semihosting-storage` | `udp-external-interface` | `run_udp_semihosting.ps1` |

Both must compile, link, and execute successfully at all times. Do not break either configuration when working on the other.

### `raft-node/` — Raft Single Node Logic

Mirrors `etheram-node/` structurally with Raft-specific types. `#![no_std]` from day one.

```rust
pub struct RaftNode<P: Clone + 'static> {
    peer_id: PeerId,
    incoming: RaftIncomingSources<P>,        // polls Timer, ExternalInterface, Transport
    state: RaftState<P>,                     // wraps Storage + Cache
    executor: RaftExecutor<P>,               // executes output actions
    context_builder: Box<dyn RaftContextBuilder<P>>,
    brain: BoxedRaftProtocol<P>,
    partitioner: Box<dyn RaftPartitioner<P>>,
    state_machine: Box<dyn RaftStateMachine>,
    observer: Box<dyn RaftObserver>,
}
```

**P** is the log entry payload type (e.g. `Vec<u8>` for serialized key-value commands).

**The step loop is identical in structure to `EtheramNode::step()`:**
1. `incoming.poll()` — get next event (transport, timer, or client)
2. `context_builder.build()` — read state into immutable `RaftContext<P>`
3. `brain.handle_message()` — pure Raft logic, returns `ActionCollection<RaftAction<P>>`
4. `partitioner.partition()` — 2-way split: `(mutations, outputs)` (no execution tier — Raft has no EVM)
5. `state.apply_mutations()` — apply storage/cache mutations
6. `apply_state_machine_outputs()` — invoke `RaftStateMachine` for `ApplyToStateMachine` actions
7. `executor.execute_outputs()` — dispatch network/timer/client I/O

**`RaftAction<P>` state mutation variants** (go to mutations bucket):
`SetTerm`, `SetVotedFor`, `AppendEntries`, `TruncateLogFrom`, `SaveSnapshot`, `AdvanceCommitIndex`, `TransitionRole`, `SetLeaderId`, `UpdateMatchIndex`, `UpdateNextIndex`

**`RaftAction<P>` output variants** (go to outputs bucket):
`SendMessage`, `BroadcastMessage`, `ScheduleTimeout`, `ApplyToStateMachine`, `SendClientResponse`, `Log`

Concrete Raft implementations, `*Variant` enums, and `*Builder` APIs live directly in `raft-node/`.

### `raft-validation/` — Raft Cluster Tests

Provides: `RaftCluster` harness with `step()`, `drain()`, `drain_all()`, `drain_except()`, `fire_timer()`, `inject_message()`, `submit_command()`, `drain_responses()`, and node state query methods. 54 cluster-level tests across election, replication, fault tolerance, snapshots, state machine, and client interface.

### `raft-embassy/` — Raft Embedded Port (Sprint 6, planned)

Will provide two configurations:

| Configuration | Transport | Storage | External Interface | Script |
|---|---|---|---|---|
| **all-in-memory** | `channel-transport` | `in-memory-storage` | `channel-external-interface` | `run_raft_channel_in_memory.ps1` |
| **real** | `udp-transport` | `semihosting-storage` | `udp-external-interface` | `run_raft_udp_semihosting.ps1` |

5-act scenario: election → replication → read-after-write → leader crash → continued replication.

---

## Naming Conventions

| Concept | Convention | Example |
|---|---|---|
| Trait (interface) | Simple noun | `Partitioner`, `ContextBuilder`, `StorageAdapter` |
| Concrete type | Descriptive prefix | `TypeBasedPartitioner`, `EagerContextBuilder`, `InMemoryStorage` |
| Boxed trait object | `Box<dyn Trait>` | `Box<dyn Partitioner>` |
| Named type alias | `Boxed` prefix | `BoxedProtocol` |
| Enum of variants | `*Variant` suffix | `StorageVariant`, `ProtocolVariant` |
| Per-component builder | `*Builder` suffix | `StorageBuilder`, `PartitionerBuilder` |

---

## Architectural Principles

### 1. Element-Centered Design
- `etheram-node/` implements a **single node** — no cluster concepts in node logic
- Peer awareness is **protocol-scoped** (validator set as parameter), never global topology knowledge
- "Cluster" lives only in `etheram-validation/`

### 2. Total Swappability
Every layer is swappable at runtime:
- **Infrastructure**: Storage, Cache, Transport, Timer, ExternalInterface
- **Decision**: ContextBuilder, Protocol (Brain), Partitioner
- **Future**: Cryptographic primitives (`SignatureScheme` trait — Ed25519 ↔ BLS ↔ Mock)

### 3. Protocol Logic is Pure
- `brain.handle_message()` takes immutable context, returns declarative actions
- No I/O in protocol logic
- Enables exhaustive testing and formal reasoning

### 4. Actions are Partitioned
- `Partitioner` separates state mutations from output effects
- `state.apply_mutations()` and `executor.execute_outputs()` are distinct
- Enforces side-effect isolation

### 5. Testing as Muscle, Not Brain
- Test by **substituting components**, not scripting scenarios
- Isolation: swap to mocks/fakes
- Adversarial: swap to Byzantine implementations
- Chaos: enumerate component combinations
- No custom scenario setup required

### 6. All In-Memory for Development
- Use `InMemoryStorage`, `InMemoryCache`, `NoOpTransport`, `ManualTimer` during development
- Zero external dependencies, instant startup, deterministic debugging
- Swap to production implementations (RocksDB, TCP, system timer) only when needed

### 7. Protocol Consistency

**This is a hard pre-feature gate. Before writing a single line of new protocol code, execute the full audit below and confirm every point is green. Skipping this audit is never acceptable.**

#### Pre-Feature IBFT Consistency Audit (run before every feature)

1. Run `cargo test -p etheram-node --test all_tests` — all protocol-level tests must pass with zero failures.
2. Run `cargo test -p etheram-etheram-validation --test all_tests` — all cluster-level tests must pass with zero failures.
3. Manually verify each invariant below against the current source before touching it:

| # | Invariant | Where to check |
|---|---|---|
| 1 | **Quorum** = `⌊2n/3⌋ + 1` | `ValidatorSet::quorum_size()` |
| 2 | **Locked-block preservation** — `pending_block` not cleared on round change when `PreparedCertificate` is set | `reset_for_new_round()` |
| 3 | **Locked-block re-propose** — proposer with cert must re-propose the locked block | `handle_timer_propose_block()` |
| 4 | **Highest-round cert wins** — incoming cert with higher round replaces current; lower or equal round is ignored | `handle_view_change()` |
| 5 | **NewView is authoritative** — `valid_new_view` guard is the sole gate; no second compat check permitted | `handle_new_view()` |

4. For any invariant whose test coverage is missing or weak, add the tests first — before writing the feature.
5. Only after all tests are green and all invariants are confirmed in source, begin the feature.

The mandatory invariants (must never be weakened by any change):
  1. **Quorum** — quorum size is `⌊2n/3⌋ + 1` (integer division). Any other formula is wrong for non-canonical validator counts.
  2. **Locked-block preservation** — when a node holds a `PreparedCertificate`, `pending_block` must not be cleared on round transitions. The locked block is preserved until a new commit occurs.
  3. **Locked-block re-propose** — a proposer entering a new round with a valid `PreparedCertificate` must re-propose the locked block (matching `cert.block_hash`), not a fresh block.
  4. **Highest-round certificate wins** — when processing `ViewChange` or `NewView` messages, an incoming `PreparedCertificate` with a higher round than the locally held one must replace it. Rejecting a certificate because it differs from the local one is a safety violation.
  5. **NewView is authoritative** — the `valid_new_view` guard is sufficient; no additional compatibility check is permitted after it passes.

### 8. Observability
- Every new protocol action, mutation, or output must be reflected in the `Observer` trait so it can be logged, traced, or asserted in tests
- Silent side-effects are not permitted

### 9. EVM Compatibility
- Every change that touches transaction execution, opcode dispatch, storage access, or account mutation must be cross-checked against the `TinyEvmEngine` and `ValueTransferEngine` implementations
- New opcodes, new storage mutation kinds, or new `ExecutionEngine` return paths must be reflected in both engines (or explicitly justified as engine-specific)
- The `ExecutionEngine` trait contract — immutable input, declarative `ExecutionResult`, no I/O — must be preserved by every change

### 10. Gas Metering Consistency
- Every new opcode added to `TinyEvmEngine` must have a corresponding constant in `tiny_evm_gas.rs` and must deduct that cost before execution inside `execute_bytecode`
- Every new transaction path in `ValueTransferEngine` must deduct `INTRINSIC_GAS` and return `OutOfGas` when `gas_limit < INTRINSIC_GAS`
- `TransactionReceipt` must be emitted for every transaction in every committed block — Success and OutOfGas alike — with correct `gas_used` and monotonically increasing `cumulative_gas_used`
- Gas constants must stay aligned with the post-Istanbul EVM schedule; deviations require an explicit comment in `tiny_evm_gas.rs`

---

## Three-Stage Validation Workflow

All three stages are **mandatory** for every new feature at the `etheram-node/` or protocol level. Do not mark a feature complete unless all three stages are satisfied.

1. **Stage 1** — Implement in `etheram-node/` → unit tests in `etheram-node/tests/` for pure data types; single-node integration tests (`EtheramNode` + concrete protocol) in `etheram-validation/tests/`
   - Logic correctness, isolated component testing
2. **Stage 2** — Validate in `etheram-validation/` → cluster tests in `etheram-validation/tests/`
   - Distributed correctness, multi-node scenarios, Byzantine fault injection
3. **Stage 3** — Exercise in `etheram-embassy/src/main.rs` → QEMU execution
   - no-std compatibility, Embassy async runtime, resource constraints
   - The test application must demonstrate the feature end-to-end (even minimally)

---

## Current Implementation Status

### ✅ Implemented
- Full 6-dimension EtheramNode with step loop
- `EtheramExecutor` with `new_with_peers()` — `SendMessage` and `BroadcastMessage` actions delivered via transport; `new()` (empty peers) preserved for test harnesses with manual message orchestration
- InMemoryStorage, InMemoryCache
- InMemoryTimer (with `schedule` and `push_event` for test driving)
- InMemoryExternalInterface (with `push_request` / `drain_responses` for test driving)
- EagerContextBuilder, TypeBasedPartitioner, NoOpProtocol
- NoOpTransport, NoOpExternalInterface
- EtheramNodeBuilder (builder pattern for node construction)
- Per-component builders with `PartitionerVariant`, `StorageVariant`, etc.
- InMemoryTransport (incoming + outgoing), shared `Arc<Mutex<InMemoryTransportState>>` per cluster
- etheram-validation cluster harness with `fire_timer`, `submit_request`, `drain_responses`, `step_all`, `push_transport_message`
- `etheram-node/tests/common_types/block_tests.rs` — 3 tests; `etheram-node/tests/common_types/account_tests.rs` — 4 tests; `etheram-node/tests/common_types/state_root_tests.rs` — 5 tests
- `etheram-node/tests/implementations/in_memory_storage_tests.rs` — 7 tests
- `etheram-node/tests/implementations/in_memory_timer_tests.rs` — 5 tests
- `etheram-node/tests/implementations/in_memory_external_interface_tests.rs` — 5 tests
- `etheram-node/tests/implementations/type_based_partitioner_tests.rs` — 3 tests
- `etheram-node/tests/implementations/in_memory_transport_tests.rs` — 5 tests
- IBFT Sprint 1: `ValidatorSet`, `VoteTracker`, `SignatureScheme` trait, `MockSignatureScheme`, `IbftMessage`, `IbftProtocol<S>` — under `etheram-node/src/implementations/ibft/`
- `IbftCluster` harness — under `etheram-validation/src/ibft_cluster.rs`
- `IbftTestNode` harness — under `etheram-validation/src/ibft_test_node.rs`
- `etheram-node/tests/implementations/ibft/validator_set_tests.rs` — 4 tests
- `etheram-node/tests/implementations/ibft/vote_tracker_tests.rs` — 5 tests
- `etheram-node/tests/implementations/ibft/mock_signature_scheme_tests.rs` — 2 tests
- `etheram-node/tests/implementations/ibft/ibft_protocol_propose_tests.rs`, `ibft_protocol_pre_prepare_tests.rs`, `ibft_protocol_prepare_tests.rs`, `ibft_protocol_commit_tests.rs`, `ibft_protocol_view_change_tests.rs`, `ibft_protocol_client_tests.rs`, `ibft_protocol_persistence_tests.rs`, `ibft_protocol_replay_tests.rs`, `ibft_protocol_validator_set_update_tests.rs`, `ibft_protocol_dedup_tests.rs`, `ibft_protocol_injection_tests.rs`, `ibft_protocol_malicious_block_tests.rs`, `ibft_protocol_signature_tests.rs`, `ibft_protocol_future_buffer_tests.rs` — per-behaviour protocol test files
- `etheram-validation/tests/etheram_node_tests.rs` — 3 single-node integration tests (Stage 1+2 bridge)
- Stage 3 skeleton (Checkpoint 1): initial `no_std` wiring — `ChannelTransportHub`, `OutboxTransport`, `ClientChannelHub` / `EtheramClient` channel API, `node_task` spawning; 5-node IBFT consensus verified via QEMU
- Stage 3 skeleton (Checkpoint 2): Async Embassy runtime — `ChannelTransportHub` (static `embassy_sync::Channel` arrays), `OutboxTransport` (sync-to-async bridge), `ClientChannelHub` + `EtheramClient` (channel-based client API), `node_task` (`#[embassy_executor::task(pool_size=5)]` with `select4`), `setup::initialize_client()` (spawns 5 async node tasks), ARM cross-compilation verified
- Stage 3 skeleton (Checkpoint 3): UDP + semihosting infra hardening — `WireIbftMessage` (postcard-serializable mirror types with `From` conversions), `UdpIbftTransport` (serialized message passing), `SemihostingStorage` (mutation-counting with ARM-gated `info!` logging), `SystickDriver` (ARM `embassy-time-driver` with SysTick exception handler), `SemihostingWriter` + `info!` macro (ARM semihosting logging), feature-matrix mutual-exclusivity guards verified
- Stage 3: `main.rs` follows Create → Start → Reach Quorum → Graceful Shutdown lifecycle; `EtheramClient::shutdown()` triggers `CancellationToken` for node task termination
- Stage 3 scenario coverage: Act 0 (IBFT warmup/height progression), Act 1 (tx commit + balance update), Act 2 (reverse transfer), Act 3 (overdraft → `InsufficientBalance`), Act 4 (view change via `TimeoutRound`), Act 5 (stale nonce → `InvalidNonce`), Act 6 (gas limit exceeded → `GasLimitExceeded`), Act 7 (validator set update at height 5 — consensus continues with updated set)
- Real `compute_state_root`: deterministic XOR-mix hash over sorted `BTreeMap<Address, Account>`; `InMemoryStorage` and `SemihostingStorage` auto-recompute on every `UpdateAccount` mutation; genesis accounts set the initial root; `EtheramState::query_state_root()` exposes the value; `EagerContextBuilder` reads from storage rather than re-computing over a partial account map
- Transaction application on commit: `IbftProtocol::handle_commit` emits `UpdateAccount` (sender balance−value, nonce+1) and `UpdateAccount` (receiver balance+value) plus `UpdateCache { RemovePending }` for each transaction in the committed block, before `StoreBlock` and `IncrementHeight`
- Extended `Observer` trait: `ActionKind` enum (non-generic projection of `Action<M>` enabling `Box<dyn Observer>` object safety); trait methods replaced — `actions_produced`/`mutations_applied`/`outputs_executed` removed; added `context_built(peer_id, height, state_root, pending_tx_count)`, `action_emitted(peer_id, &ActionKind)`, `mutation_applied(peer_id, &ActionKind)`, `output_executed(peer_id, &ActionKind)`; `EtheramNode::step` calls each per-item; `SemihostingObserver` logs per-item detail at appropriate levels
- Embassy 7-act scenario: genesis accounts seeded (`sender=[1u8;20]` balance 1000, `receiver=[2u8;20]` balance 200) via `NodeInfraSlot::with_genesis_account`; `EtheramClient::submit_to_all_nodes` broadcasts tx to all 5 nodes (proposer always has the tx regardless of round); Act 1: transfer 300 → balances 700/500; Act 2: reverse 200 → 900/300; Act 3: overdraft 400 → `InsufficientBalance`; Act 4: `TimeoutRound` quorum → view change → height increments; Act 5: stale nonce (nonce=0 after it was already used) → `InvalidNonce`; Act 6: gas_limit=1_000_001 > `MAX_GAS_LIMIT` → `GasLimitExceeded`; Act 7: `ValidatorSetUpdate` scheduled at height 5 in both configurations — consensus continues through the transition
- `EtheramClient` cfg-free facade: feature-specific dispatch pushed into `infra/external_interface/client_facade.rs`; public functions (`submit_ei_request`, `submit_ei_to_all_nodes`, `await_ei_response`) are unconditional and delegate to private cfg-gated helpers (`submit_impl`, `await_impl`); `etheram_client.rs` contains zero `#[cfg(...)]` attributes
- Commit signatures: `IbftMessage::Commit` carries `sender_signature: SignatureBytes`; `commit_commitment_payload()` (prefix `2`, height+round+block_hash LE) verified via `SignatureScheme::verify_for_peer()` in `valid_commit()`; `WireIbftMessage::Commit` updated for UDP serialization
- Future-round message buffer: `IbftProtocol` buffers `PrePrepare`/`Prepare`/`Commit` messages for rounds ahead of `current_round` (up to `MAX_FUTURE_BUFFER_SIZE = 100`); buffering occurs before `accept_peer_message()` to avoid polluting dedup state; replay is triggered in `handle_message()` when `current_round` advances (via `TimeoutRound`, `ViewChange`, or `NewView`)
- `ValidatorSet.faulty_count` removed (dead code — quorum is computed directly from validator count)
- `TinyEvmEngine` unknown opcode returns `OutOfGas` (was `Success`)
- `StoreReceipts` storage mutation kind computes real success/out_of_gas counts from receipt statuses

### ✅ E1 — TLA+ Formal Specifications
- IBFT: `IBFTConsensus.tla` — Agreement, LockConsistency, CommitImpliesPrepareQuorum, Termination; Byzantine fault model (`ByzValidators`). Quick ~5s (exit 0), CI ~21s (exit 0)
- Raft: `RaftConsensus.tla` — ElectionSafety, VoteOnce, LeaderTermOK, LogSafety, LeaderCompleteness; log replication + stale-leader scenarios. Quick 1.6s / 1175 states (exit 0), CI 10.7s / 282K states (exit 0)
- Liveness (`Termination`) defined via `FairSpec` for manual verification; swap `SPECIFICATION Spec` → `FairSpec` and add `PROPERTIES Termination` in the cfg to activate
- Scripts: `scripts/ibft_run_tla_quick.ps1`, `scripts/raft_run_tla_quick.ps1` — **never invoked automatically by an AI agent**

### 🔄 Next: Ethereum Functionalities — C3 → C6 → C1
- Detailed implementation plan: [C-Functionalities.md](etheram-node/C-Functionalities.md)
- **C3** — Transaction pool priority ordering: add `gas_price: u64` to `Transaction`; replace `Vec<Transaction>` in `InMemoryCache` with a `BTreeSet` ordered by `(gas_price DESC, nonce ASC, from ASC)`; pool capacity 4096 with lowest-priority eviction; per-sender nonce deduplication; `ZeroGasPrice` rejection reason; ordering validated in `valid_pre_prepare`
- **C6** — Block gas limit: add `gas_limit: Gas` to `Block`; `BLOCK_GAS_LIMIT = 10_000_000`; proposer greedy-fills from sorted pool up to the limit; `valid_block_gas()` rejects over-limit or non-canonical-limit blocks
- **C1** — Expand TinyEVM: 22 new opcodes (MSTORE/MLOAD, CALLDATALOAD/CALLDATASIZE, SHA3, JUMP/JUMPI, PUSH2–32, DUP1–16, SWAP1–16, POP, arithmetic, CALLER, CALLVALUE, REVERT); memory model with quadratic expansion gas; JUMPDEST pre-scan; `tiny-keccak` dependency
- **Three implementation traps to review before coding:**
  1. **JUMPDEST pre-scan** — PUSH2 consumes 2 immediate bytes that must be skipped during the scan, not treated as opcodes. Off-by-one here corrupts all JUMP targets.
  2. **Memory expansion gas** — charge the *delta* (`cost(new_high_water) − cost(old_high_water)`), not the absolute cost per access. Charging absolute cost per MLOAD is ruinously expensive and wrong.
  3. **C3 pool eviction tie-breaking** — when gas prices are equal the eviction invariant must be explicit: lowest priority = lowest gas price, then highest nonce, then highest `from` (reverse lexicographic). Ambiguity here causes non-deterministic block ordering across nodes.

### ✅ Raft Sprints 0–5 Implemented
- `raft-node/` crate created with `#![no_std]` from day one
- All Sprint 0 types: `RaftMessage<P>` (8 variants), `RaftAction<P>` (17 variants inc. `RestoreFromSnapshot`), `RaftContext<P>`, `RaftTimerEvent`, `RaftClientRequest`, `RaftClientResponse`, `RaftStorageQuery`, `RaftStorageMutation<P>`, `RaftStorageQueryResult<P>`, `RaftCacheQuery`, `RaftCacheUpdate`, `RaftCacheQueryResult`, `NodeRole`, `LogEntry<P>`, `RaftSnapshot`, `RaftStateMachine` trait
- Sprint 1 `RaftNode<P>` with full 6-dimension struct and step loop matching `EtheramNode::step()` structure
- `RaftObserver` trait with `RaftActionKind` projection (inc. `RestoreFromSnapshot`), `action_kind()` helper
- `RaftPartitioner<P>` producing 2-way partition (mutations, outputs) — no execution tier
- All adapter blanket impls: `StorageAdapter<P>`, `CacheAdapter`, `TimerInputAdapter`, `TimerOutputAdapter`, `TransportIncomingAdapter`, `TransportOutgoingAdapter`, `ExternalInterfaceIncomingAdapter`, `ExternalInterfaceOutgoingAdapter`
- `RaftIncomingSources<P>`, `RaftOutgoingSources<P>`, `RaftExecutor<P>` with poll and execute loops
- Sprint 2 `RaftProtocol<P>` in `raft-node/` — pure Raft consensus: pre-vote, election, leader promotion, heartbeat, log replication, snapshot install; `ELECTION_TIMEOUT_MS=300`, `HEARTBEAT_INTERVAL_MS=100`; quorum = `(n+1)/2 + 1`
- Sprint 3 infra implementations in `raft-node/`: `InMemoryRaftStorage<P>`, `InMemoryRaftCache`, `InMemoryRaftTransport<P,S>`, `InMemoryRaftTimer<S>`, `InMemoryRaftExternalInterface<S>`, `InMemoryRaftStateMachine`, `NoOpRaftTransport<P>`, `NoOpRaftObserver`, `TypeBasedRaftPartitioner`, `EagerRaftContextBuilder`, `SharedState<T>` trait; `RaftNodeBuilder<P>` builder
- Sprint 4 tests in `raft-node/tests/`: 42 protocol-level tests across `election_tests`, `replication_tests`, `snapshot_tests`, `client_tests`, `role_transition_tests`, `in_memory_raft_storage_tests`, `in_memory_raft_cache_tests` — all passing
- Sprint 5 cluster tests in `raft-validation/tests/`: 54 cluster-level tests across `election_tests`, `replication_tests`, `fault_tolerance_tests`, `snapshot_tests`, `state_machine_tests`, `client_tests` — all passing; `RaftCluster` harness with `step()`, `drain()`, `drain_all()`, `drain_except()`, `fire_timer()`, `inject_message()`, `submit_command()`, `drain_responses()`; pre-flight fixes to `raft_node.rs` (`state()` + `peer_id()` accessors, `P: AsRef<[u8]>` bound, correct state machine payload), `common.rs` (`SendClientResponse` now emitted in `advance_commit_index`), and `in_memory_raft_timer.rs` (`schedule()` is a no-op — test harness drives timer events explicitly)

---

## Key Constraints

- `etheram-node/` must compile with no cluster-level dependencies
- Protocol logic must remain pure (no I/O)
- **Circular dependencies are forbidden** — no crate may directly or transitively depend on itself, including via `[dev-dependencies]`. Before adding any dependency between crates, verify the full dependency chain contains no cycle.
- **Dependency direction is one-way** — `etheram-validation` / `etheram-embassy` may depend on `etheram-node`; `etheram-node` must never depend on validation/embassy crates.
- All new swappable components need a corresponding `*Variant` enum entry and `*Builder` in `etheram-node/` (and analogously in `raft-node/`).
- Trait names: simple nouns. Concrete names: descriptive prefixes
- `core`, `etheram-node`, and `raft-node` must be `no_std`-compatible — they carry `#![no_std]` and use `alloc` for heap types (`Box`, `Vec`, `String`, `BTreeMap`). No `std`-only types or imports are permitted in these crates
- `etheram-embassy/` must remain `no_std`-compatible
- **`etheram-embassy/` must always maintain both configurations** — the all-in-memory configuration (`channel-transport` + `in-memory-storage` + `channel-external-interface`) and the real configuration (`udp-transport` + `semihosting-storage` + `udp-external-interface`) must both compile, link, and run at all times. Every change must be verified against both feature sets before marking complete.
- **Workspace dependency governance is mandatory** — all dependency versions/features and all local crate links must be declared in the workspace root `Cargo.toml` under `[workspace.dependencies]`. Member crates must reference them via `.workspace = true` and must not declare per-crate `path =`, version, or feature overrides for those dependencies. The only allowed `path =` entries outside root dependency declarations are target declarations such as `[lib] path`, `[[bin]] path`, and `[[test]] path`.
- **The raft crate family is independent from etheram** — `raft-node/`, `raft-validation/`, and `raft-embassy/` depend only on `core/` and `raft-node` as appropriate. No `raft-*` crate may import from `etheram*` and no `etheram*` crate may import from `raft-*`. Cross-dependencies between protocol families are forbidden.
- **Raft dependency direction mirrors etheram** — `raft-validation` / `raft-embassy` may depend on `raft-node`; `raft-node` must never depend on validation/embassy crates. Tests in `raft-node/tests/` can only use what `raft-node` itself exposes.

---

## Coding Style Rules

- **File header** — every Rust source file must begin with the Apache 2.0 copyright header:
  ```rust
  // Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
  // Licensed under the Apache License, Version 2.0
  // http://www.apache.org/licenses/LICENSE-2.0
  ```
- **No comments in production code** — code should be self-explanatory through naming and structure. The only permitted comments are `// TODO:` and `// FIXME:` (with a description). Doc comments (`///` and `//!`) are not allowed.
- **No unit tests unless explicitly requested** — do not add or modify unit tests by default. In particular, do not add inline test modules (`#[cfg(test)] mod tests { ... }`) in `src/` files.
- **Integration-test-first policy** — when tests are requested or clearly appropriate, add integration tests under `tests/` (not inline in `src/`), and within reason structure production code so integration tests can be added cleanly.
- **Single-primary-type per file (default)** — each production Rust file should define one primary `struct` or `enum`, and the filename should match that primary type in snake_case. Additional `struct`/`enum` items are allowed only when tightly coupled and small (for example helper DTOs/wire mirrors, companion conversion wrappers, or context carrier pairs). When a file grows or carries multiple independent responsibilities, split it.
- **Prefer methods over free functions** — all behaviour associated with a `struct` or `enum` must live in an `impl` block as methods. Avoid standalone free functions that operate on a struct's data. Free functions are acceptable only when they are truly stateless utilities with no struct affinity (e.g. pure converters, top-level entry points).
- **Always use `use` imports** — never write inline path segments in function signatures, return types, or expressions (e.g. `etheram_node::incoming::timer::timer_event::TimerEvent`). Every type used in code must appear in a `use` declaration at the top of the file.
- **`use` blocks are compacted and sorted** — all `use` statements must be grouped together with no blank lines between them, and sorted alphabetically. This must be verified before completing any task.
- **1 empty line after file header** — there must be exactly one blank line between the 3-line Apache 2.0 copyright header and the first `use` statement.
- **1 empty line between every code block** — there must be exactly one blank line between every top-level code block: between the `use` block and the first item, and between any two consecutive items (struct, enum, trait, impl, fn). No double blank lines.
- **No code in `mod.rs` files** — `mod.rs` (and `lib.rs`) must contain only `pub mod` declarations. All productive code belongs in a dedicated file named after its primary type or concern. Consumers import directly via the full path:
  ```rust
  // common/mod.rs
  pub mod test_node;

  // node_step.rs
  mod common;
  use crate::common::test_node::TestNode;
  ```
- **Remove mod-only folders** — if a folder contains only `mod.rs` and no other files/subfolders, flatten it by moving/removing the module declaration and deleting the redundant folder.
- **Tests follow AAA** — every test must be structured in three labelled sections: `// Arrange`, `// Act`, `// Assert`. When act and assert collapse into a single expression, use `// Act & Assert`. No other comments are permitted in test bodies. Each section must be separated from the next by exactly one blank line.
- **Run `cargo fmt` after every change** — always run `cargo fmt` from the workspace root after editing any Rust source file.
- **No warnings** — the codebase must compile with zero warnings. Every unused import, dead code path, or missing trait implementation that triggers a compiler warning must be fixed before committing. `#[allow(...)]` attributes are not permitted except for `#[allow(clippy::too_many_arguments)]` on builder constructors.
- **Mandatory pre-feature IBFT audit** — before writing any new protocol feature that touches `IbftProtocol`, `ValidatorSet`, `VoteTracker`, or any handler in `ibft_protocol*.rs`, execute all five steps of the Pre-Feature IBFT Consistency Audit in Architectural Principle 7. Both `cargo test -p etheram-node --test all_tests` and `cargo test -p etheram-validation --test all_tests` must be green, and every invariant must be confirmed in source, before the first line of the new feature is written. This is a hard gate — not a suggestion.
- **Mandatory pre-feature Raft audit** — before writing any new protocol feature that touches `RaftProtocol`, run `cargo test -p raft-node --test all_tests` and `cargo test -p raft-validation --test all_tests` and confirm the following Raft invariants hold in source before beginning: (1) quorum = `⌊n/2⌋ + 1`; (2) leader never appends entries in a term other than its own; (3) voted_for is persisted before sending RequestVoteResponse; (4) commit_index only advances when a majority have acknowledged the entry; (5) step-down occurs immediately on receiving any message with a higher term. This is a hard gate — not a suggestion.
- **Run tests before marking complete** — always run the appropriate quality gate before considering any task done:
  - **Documentation-only exception** — when the change is limited to documentation files only (`*.md` and similar doc-only content) and no code, tests, scripts, or configuration files changed, no quality gate is required.
  - **Minimal gate (no productive code changed)** — run `powershell -File scripts\run_tests.ps1`. Use this when the change is limited to test files, scripts, or configuration only.
  - **Full gate (productive code changed)** — run `powershell -File scripts\run_tests.ps1` first, then `powershell -File scripts\run_apps.ps1`. Use this whenever any file under a `src/` directory is added, edited, or deleted. Both must exit with code 0. `scripts\test.ps1` is a convenience wrapper that runs both sequentially.
  - **Direction H fast-path exception** — when productive changes are limited to `etheram-node-process/src/**` (and optional `etheram-desktop/src/**`) with no changes under `etheram-embassy/` or `raft-embassy/`, `powershell -File scripts\run_apps.ps1` is not required. In this case, run `powershell -File scripts\run_tests.ps1` and the mandatory no_std checks only.
  - In all cases, all required scripts for the selected gate must exit with code 0 before a task is marked complete.
- **Mandatory no_std gate for core + embassy-core + nodes** — every completion gate must include explicit checks for all four crates: `cargo check -p etheram-core --no-default-features`, `cargo check -p embassy-core --no-default-features`, `cargo check -p etheram-node --no-default-features`, and `cargo check -p raft-node --no-default-features`.
- **Mandatory Raft no_std gate** — when working on `raft-node/`, always run an explicit no_std compatibility check: `cargo check -p raft-node --no-default-features`.
- **Mandatory dual-layer test updates for Raft productive changes** — every time productive Raft code is added, changed, or fixed, update tests in both layers: protocol-level tests in `raft-node` and cluster-level tests in `raft-validation`. The same rule applies as for etheram: do not mark work complete unless both layers are updated or explicitly justified as not applicable.
- **Raft–etheram consistency check** — whenever productive code or tests are added or changed in either protocol family, check the parallel artefact in the other family for inconsistencies. This includes: structural divergence in equivalent types (e.g. `RaftNode` vs `EtheramNode`, `RaftPartitioner` vs `Partitioner`, `RaftObserver` vs `Observer`), naming convention drift, step-loop shape differences, test organisation deviations, and missing analogous tests. Inconsistencies that are intentional (protocol-specific types, Raft 2-way vs etheram 3-way partition) must be explicitly justified in a `// FIXME:` or `// TODO:` comment. Silent drift is not permitted.
- **Mandatory Stage 3 no_std gate for node crate** — when working on Stage 3 (`etheram-embassy/`), always run an explicit no_std compatibility check for `etheram-node`: `cargo check -p etheram-node --no-default-features`.
- **Stage 3 test application never sleeps** — `main.rs` must not use fixed-duration sleeps (`Timer::after`) to wait for consensus or protocol progress. Use `EtheramClient::wait_for_height_above` (or an equivalent polling helper with a timeout ceiling) instead. Fixed sleeps are only permitted for non-observable housekeeping (e.g. a brief shutdown drain).
- **Mandatory dual-layer test updates for productive changes** — every time productive code is added, changed, or fixed, update tests in both layers: protocol-level tests in `etheram-node` and cluster-level tests in `etheram-validation`. Do not mark work complete unless both layers are updated or explicitly justified as not applicable.
- **Mandatory test deduplication across files** — when identical or near-identical test setup/logic appears more than twice across test files in the same crate, refactor it into shared test helpers and update all affected files to remove duplication.
- **Property-based tests with proptest** — both protocol families maintain a `prop_tests/` subfolder mirroring each other under `tests/implementations/ibft/` and `tests/implementations/raft_protocol/`. Each subfolder contains three files: `client_proptest_tests.rs`, `peer_message_proptest_tests.rs`, and `timer_proptest_tests.rs`. Every new protocol handler or client-facing path must have at least one corresponding proptest property verifying its output contract holds across arbitrary valid inputs. Use `ProptestConfig::with_cases(200)`. The `proptest = "1"` dev-dependency is declared in `[workspace.dependencies]` and referenced via `.workspace = true` in each node crate's `[dev-dependencies]`.
- **No `#[path = "..."]` module imports in tests** — do not import test modules using path attributes. Add the helper module to the corresponding `mod.rs` and import it through the normal module tree.
- **Method input/output contract** — methods must take immutable input parameters (`&T` for borrowed inputs), return computed results, and must not mutate input parameters. Do not use mutable out-parameters (for example `&mut Vec<_>`, `&mut Option<_>`, `&mut ActionCollection<_>`) to return data. Allowed mutation is limited to receiver state (`&mut self`) and local variables.
- **Test file naming** — test files are named `<StructName>_tests.rs` (snake_case of the primary struct under test), e.g. `etheram_node_tests.rs` for `EtheramNode`.
- **Test function naming** — `<method_under_test>_<scenario>_<expected_result>`, e.g. `step_empty_queues_returns_false`, `query_account_genesis_account_returns_balance`.
- **Test folder mirrors source folder** — the `tests/` directory must mirror the structure of `src/` exactly. For each subdirectory in `src/`, there is a matching subdirectory in `tests/` with a `mod.rs` that lists only `pub mod` declarations. The test root is `tests/all_tests.rs`, which is the single integration test entry point. Example:
  ```
  src/implementations/ibft/validator_set.rs
  src/implementations/ibft/vote_tracker.rs

  tests/all_tests.rs                                         ← declares: pub mod implementations;
  tests/implementations/mod.rs                               ← declares: pub mod ibft; pub mod ...;
  tests/implementations/ibft/mod.rs                          ← declares: pub mod validator_set_tests; pub mod vote_tracker_tests;
  tests/implementations/ibft/validator_set_tests.rs
  tests/implementations/ibft/vote_tracker_tests.rs
  ```
- **Direction H crate bootstrap rule** — `etheram-desktop/` and `etheram-node-process/` must be created with a `tests/` folder and `tests/all_tests.rs` from day one. Every new production module added under `src/` during Direction H work must either add a corresponding integration test in `tests/` in the same task or include an explicit justification for deferral.
- **Direction H dependency boundary** — `etheram-desktop` may depend on `etheram-core`, `etheram-node`, and `etheram-node-process` (plus external ecosystem crates). `etheram-node-process` may depend on `etheram-core` and `etheram-node` only (plus external ecosystem crates). Neither may depend on `raft-*` crates.

---

## TLA+ Formal Specifications

The `specs/` directory contains formal TLA+ model specifications for both protocol families. Each specification is checked with TLC (the TLA+ model checker).

### IBFT (`specs/ibft/`)
- `IBFTConsensus.tla` — core specification with Byzantine fault model (`ByzValidators`, `CorrectValidators`, Byzantine injection actions, invariants scoped to correct validators)
- `MC_IBFTConsensus_Quick.tla` — parametric override for quick check (MaxRound=0, ByzValidators={3}, ~5s)
- `MC_IBFTConsensus_CI.tla` — parametric override for honest baseline (MaxRound=1, ByzValidators={}, ~21s)
- `MC_IBFTConsensus.tla` — parametric override for full check (MaxRound=2, ByzValidators={3}, 30+ minutes)

### Raft (`specs/raft/`)
- `RaftConsensus.tla` — election safety specification (invariants: `ElectionSafety`, `VoteOnce`, `LeaderTermOK`)
- `MC_RaftConsensus_Quick.tla` — parametric override for quick check (N=3, MaxTerm=1, ~2s)
- `MC_RaftConsensus_CI.tla` — parametric override for CI check (N=3, MaxTerm=2, ~5s)
- `MC_RaftConsensus.tla` — parametric override for full check (N=3, MaxTerm=3, 10-30 minutes)

### Scripts

| Script | Protocol | Scope | Typical duration |
|---|---|---|---|
| `scripts/ibft_run_tla_quick.ps1` | IBFT | Byzantine (MaxRound=0) + Honest (MaxRound=1) | ~30s |
| `scripts/ibft_run_tla_full.ps1` | IBFT | Byzantine MaxRound=2 | 30+ min |
| `scripts/raft_run_tla_quick.ps1` | Raft | Election safety N=3 MaxTerm=1+2 | ~10s |
| `scripts/raft_run_tla_full.ps1` | Raft | Election safety N=3 MaxTerm=3 | 10-30 min |

### AI agent restriction

**TLA+ model checking scripts are NEVER invoked automatically by an AI agent.** These scripts start a JVM process, may consume significant CPU for minutes to hours, and produce multi-gigabyte state databases. They must only be run manually by a developer who has explicitly decided to do so. An AI agent must not call `ibft_run_tla_quick.ps1`, `ibft_run_tla_full.ps1`, `raft_run_tla_quick.ps1`, or `raft_run_tla_full.ps1` as part of any automated quality gate, task completion check, or proactive verification step — even if the user asks the agent to "run all checks" or "verify everything".
