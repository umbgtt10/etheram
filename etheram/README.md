# EtheRAM: Research-grade blockchain node implementation designed to validate architectural abstractions for distributed systems, not to compete with production Ethereum clients

The goal of this project is to explore whether a blockchain node can be cleanly decomposed into **pure protocol logic**, **explicit state dimensions**, and a **runtime-agnostic coordination layer**, such that entire subsystems (consensus, storage, execution model, transport) can be swapped without rewriting business logic.

EtheRAM intentionally targets a **minimal but non-trivial subset of Ethereum semantics**. It includes enough protocol, state, and execution complexity to invalidate toy designs, while deliberately excluding features whose implementation would add surface area without yielding new architectural insight.

This project prioritizes:
- **Separation of concerns over feature completeness**
- **Testability over throughput**
- **Explicit abstractions over ad-hoc integration**
- **Correctness and reasoning over premature optimization**

The implementation is guided by the **3-6 architectural model**, which structures a node as:
- three layers (Protocol, State, Coordination)
- across six orthogonal dimensions (Protocol, Storage, Cache, Transport, External Interface, Timer)

All protocol logic is expressed as **pure computation**: given an immutable context and an input event, the protocol produces declarative actions, which are executed separately. This enables exhaustive testing, property-based validation, and clear reasoning about system behavior independent of runtime or infrastructure.

EtheRAM is **not**:
- a full Ethereum client
- a production-ready blockchain
- an attempt to optimize gas, throughput, or latency

It **is**:
- a concrete validation of architectural ideas under real-world complexity
- a testbed for swappable execution models and state backends
- a demonstration of how far principled abstraction can be pushed in Rust
- a learning artifact for engineers interested in distributed systems, consensus, and protocol design

If the architecture survives the demands of EtheRAM’s execution and state model, it is considered validated. If it does not, the failure is considered a result, not a bug.

## Vision

EtheRAM is a **minimal Ethereum implementation with Byzantine Fault Tolerance** that validates the six-dimension Node architecture under real-world constraints:

- **Byzantine consensus** in memory-constrained environments
- **Ethereum semantics** (accounts, state roots, gas, merkle trees)
- **Dual architectural views** (6 dimensions + 3 spaces)
- **Embedded deployment** (no-std + Embassy on real hardware)

**What makes this unique:** Validates that production blockchain concerns (BFT, Ethereum state, smart contracts) work within the six-dimension architecture on embedded hardware.

---

## Architectural Goals

### The 6-Dimension View (Structural)

What components exist:

1. **Protocol** - Byzantine consensus + Ethereum state transitions
2. **Storage** - Persistent state (accounts, blocks, state roots)
3. **Cache** - Volatile working state (mempool, pending votes)
4. **Transport** - UDP peer-to-peer with unreliable delivery
5. **ExternalInterface** - Client RPC (submit tx, query balance)
6. **Timer** - Consensus timeouts, block proposals

### The 3-Space View (Functional)

What roles components play:

1. **Brain Space**
   - `build_context()` - Query Storage + Cache for current state
   - `handle_message()` - Pure protocol logic (Context + Message → Actions)
   - `execute_actions()` - Apply actions to dimensions

2. **Scheduler Space**
   - Event selection strategy (poll dimensions, choose next event)
   - Execution modes: Sequential (testing), Async (Embassy)
   - Independent of protocol logic

3. **Dimension Space**
   - **Data dimensions**: Storage, Cache (`&mut self` for mutations)
   - **I/O dimensions**: Transport, ExternalInterface, Timer (`&self` with interior mutability)

### Goal: Formalize the Relationship

Prove how the 6 dimensions map into the 3 spaces:
- Brain queries/commands dimensions
- Scheduler coordinates dimension polls
- Dimensions provide state (data) and events (I/O)

**Success**: Document the complete mapping with formal boundaries.

---

## Minimal Targets (Non-negotiable)

### Phase 0: Project Skeleton (1 week)
- Minimal abstractions leveraging TinyChain + Embassy learnings
- Hooking points for consensus, Ethereum state, dimensions
- Zero functionality, just architecture skeleton
- **Proof**: Compiles, has clear boundaries

### Phase 1: Consensus Foundation (3-4 weeks)
- Simplest BFT consensus algorithm (single-shot or PBFT-lite)
- Pure state machine, zero I/O coupling
- **Test harness with adversarial control**:
  - Byzantine nodes sending conflicting messages
  - Message reordering and delays
  - Network partitions (minority/majority)
- **Proof**: 100% deterministic consensus under all Byzantine scenarios

### Phase 2: Ethereum Brain (4-6 weeks)
- Minimal Ethereum semantics:
  - Accounts with balances and nonces
  - Transactions with gas limits
  - State roots (merkle trees)
  - Block structure with receipts
- Integrate BFT consensus with Ethereum state transitions
- **Test harness for complete node**:
  - Byzantine nodes in Ethereum context
  - Double-spend attempts
  - Invalid state transitions
  - Network partitions during transaction processing
  - Lazy scheduler
- **Proof**: BFT Ethereum node works in simulation

### Phase 3: Formalize 6-3 Model (2 weeks)
- Document how 6 dimensions map to 3 spaces
- Document abstract scheduler pattern
- Show how Brain/Scheduler/Dimensions interact
- **Implement dual builder APIs**:
  - **3-Space Builder**: `BlockchainNodeBuilder::new().with_state(...).with_brain(...).with_executor(...)`
  - **6-Dimension Builder**: `BlockchainNodeBuilder::new().with_protocol(...).with_storage(...).with_cache(...).with_transport(...).with_external_interface(...).with_timer(...)`
  - **Cluster Builder**: `BlockchainClusterBuilder::new().with_node(3_space_node).with_node(6_dim_node)`
- **Proof**: Both builder APIs produce equivalent nodes, proving view equivalence

### Phase 4: Embassy Deployment (3-4 weeks)
- Port to no-std + Embassy
- 5-node cluster on real hardware (RP2040 or STM32)
- UDP transport over WiFi/Ethernet
- Memory constraints: < 256KB per node
- Reproduce Byzantine scenarios from test harness
- **Proof**: Physical cluster reaches consensus under Byzantine faults

### Phase 5: Dimensional Independence (2 weeks)
- Swap cryptographic algorithm (ECDSA → Ed25519 OR Blake2b → SHA3)
- Zero changes outside crypto dimension
- **Proof**: Algorithms are plugin-compatible, validating dimension boundaries

---

## Nice-to-Have Targets

### Smart Contract Support (3-4 weeks)
- Minimal EVM or WASM runtime
- Contract deployment via transactions
- Contract execution in sandboxed environment
- Gas metering and state updates
- Test on Embassy hardware
- **Proof**: Programmable blockchain on embedded devices

---

## Critical Path

```
Phase 0: Skeleton (1 week)
├─ Minimal abstractions
├─ Hooking points for all dimensions
├─ Clear architectural boundaries
└─ Compiles, ready for consensus

Phase 1: Consensus (3-4 weeks)
├─ Simplest BFT algorithm
├─ Adversarial test harness
├─ All Byzantine cases covered
└─ 100% deterministic

Phase 2: Ethereum (4-6 weeks)
├─ Accounts, state roots, gas
├─ BFT + Ethereum integration
├─ Full node test harness
└─ Byzantine Ethereum validated

Phase 3: Formalization (2 weeks)
├─ 6-3 model relationship documented
├─ Abstract scheduler pattern documented
└─ Architecture fully specified

Phase 4: Embassy (3-4 weeks)
├─ No-std + Embassy port
├─ 5-node physical cluster
├─ UDP over real network
└─ Byzantine scenarios reproduced

Phase 5: Validation (2 weeks)
├─ Crypto algorithm swap
├─ Heterogeneous cluster test
└─ Independence proven

Nice-to-have: Smart Contracts (3-4 weeks)
├─ EVM/WASM runtime
├─ Contract deployment + execution
└─ Works on Embassy
```

**Total: 15-21 weeks minimum, 18-25 weeks with smart contracts**

---

## What This Proves (That Nothing Else Does)

✅ **BFT consensus in constrained environments**
- Not just "blockchain on embedded" - actual Byzantine fault tolerance
- Memory limits force architectural discipline

✅ **Real-world blockchain semantics**
- Ethereum state machine, not toy examples
- Merkle trees, gas metering, nonces
- If it handles Ethereum, it handles any blockchain

✅ **Dual architectural views validated**
- 6 dimensions (structural) proven in TinyChain/Embassy
- 3 spaces (functional) formalized here
- Relationship between both views documented
- **Builder API equivalence**: Both 3-space and 6-dimension builders produce identical nodes

✅ **Dimensional independence proven**
- Crypto swap test validates boundaries are real
- Heterogeneous clusters (different crypto per node) possible

✅ **Smart contracts on embedded** (if nice-to-have achieved)
- First programmable blockchain on embedded hardware
- Proves architecture handles full complexity

---

## Design Principles

### 1. Consensus Before Blockchain
Validate BFT algorithm in isolation before adding Ethereum complexity.

**Why:** Consensus is the hardest part. Get it right first.

### 2. Test Harness as First-Class Citizen
Adversarial, deterministic tests are not an afterthought - they're the primary validation tool.

**Why:** Byzantine scenarios are impossible to debug without full control.

### 3. Minimal Ethereum, Not Toy Blockchain
Real Ethereum data structures force real constraints.

**Why:** Toy examples hide architectural flaws.

### 4. Formalization Comes After Implementation
Document the 6-3 relationship after seeing it work, not before.

**Why:** Premature abstraction from 2 examples (TinyChain + Embassy) is dangerous.

### 5. Physical Hardware as Proof
QEMU simulation is not enough - real hardware with real network validates portability.

**Why:** Embassy async on QEMU can hide timing/memory issues.

---

## Success Criteria

### Minimum Viable Success
- 5-node cluster on physical hardware (RP2040/STM32)
- 2 Byzantine nodes (f=2, n=2f+1=5)
- UDP over WiFi with packet loss
- Cluster reaches consensus on Ethereum state
- Memory usage < 256KB per node
- All Byzantine test cases reproducible
- Crypto algorithm swapped without architectural changes
- **Both builder APIs work**: Nodes built via 3-space and 6-dimension patterns are equivalent

**This proves the core thesis.**

### Aspirational Success
All of minimum viable PLUS:
- Smart contracts execute on embedded hardware
- EVM or WASM runtime fits in memory budget
- Contract deployment + execution tested on physical cluster

**This proves production viability.**

---

## What to Absolutely Avoid

❌ **Premature optimization** - No performance tuning until functionality works
❌ **Scope creep** - ONE BFT algorithm, not multiple
❌ **Generic abstractions** - Ethereum-specific is better than blockchain-generic
❌ **Perfect replay** - Event logs just need to be correct, not efficient
❌ **Crypto bikeshedding** - Pick one signature scheme, move on

---

## Related Projects

### Learnings Applied from:

**TinyChain** ([../examples/tinychain](../examples/tinychain/)):
- 6-dimension decomposition validated
- Adversarial test harness patterns
- Static instantiation (zero-cost abstractions)
- Sequential execution primitive

**Embassy-Async** ([../examples/embassy-async](../examples/embassy-async/)):
- Async execution on no-std
- Dynamic dimension injection (trait objects)
- Event-driven architecture
- Embassy runtime integration

**Key differences:**
- EtheRAM adds Byzantine consensus (neither example has this)
- EtheRAM uses real Ethereum semantics (not toy blockchain)
- EtheRAM formalizes 6-3 model relationship (new contribution)
- EtheRAM targets no-std + Embassy + UDP + Semihosting for QEMU validation

---

## Current Status

**Phase 0: Project Skeleton** - In Progress

Creating minimal abstractions with clear hooking points:
- Dimension trait boundaries
- Brain space components (build_context, handle_message, execute_actions)
- Scheduler interface
- Test harness infrastructure

**Next:** Once skeleton is complete, implement simplest BFT consensus algorithm with full test coverage.

---

## Development Approach

### 1. Skeleton First (Current)
- Define trait boundaries
- Create hooking points
- Zero functionality
- **Goal:** Clear architecture, compiles

### 2. Consensus in Isolation
- Pure state machine
- No I/O coupling
- Test harness first
- **Goal:** Byzantine consensus proven

### 3. Ethereum Integration
- Add state machine
- Integrate with consensus
- Expand test harness
- **Goal:** BFT Ethereum in simulation

### 4. Embassy Port
- No-std translation
- Real UDP stack
- Real persistency with Semihosting
- **Goal:** QEMU validation

### 5. Formalization
- Document 6-3 mapping
- Implement dual builder APIs (3-space vs 6-dimension)
- Implement cluster builder (heterogeneous nodes)
- Extract patterns
- Write ADRs
- **Goal:** Architecture specified with builder equivalence proven

---

## Builder API Design (Phase 3 Validation)

### The Dual Builder Proof

If the 6-dimension and 3-space models truly represent the same architecture from different perspectives, then **both builder APIs should be possible and produce equivalent nodes**.

### 3-Space Builder API

Building from functional perspective (what roles components play):

```rust
let node = BlockchainNodeBuilder::new()
    .with_state(
        StateBuilder::new()
            .with_storage(InMemoryStorage::new())
            .with_cache(InMemoryCache::new())
            .build()
    )
    .with_brain(
        BrainBuilder::new()
            .with_protocol(EtheramProtocol::new())
            .build()
    )
    .with_executor(ExecutorBuilder::new().build())
    .with_scheduler(SequentialScheduler::new())
    .build();
```

**Semantic grouping:**
- **State**: Storage + Cache (data dimensions)
- **Brain**: Protocol logic (handle_message, build_context, execute_actions)
- **Executor**: Action application strategy
- **Scheduler**: Event selection strategy

### 6-Dimension Builder API

Building from structural perspective (what components exist):

```rust
let node = BlockchainNodeBuilder::new()
    .with_protocol(EtheramProtocol::new())
    .with_storage(InMemoryStorage::new())
    .with_cache(InMemoryCache::new())
    .with_transport(UdpTransport::new())
    .with_external_interface(RpcInterface::new())
    .with_timer(EmbassyTimer::new())
    .build();
```

**Flat dimension list:**
- Each dimension configured independently
- No semantic grouping
- Direct component substitution

### Cluster Builder API

Building heterogeneous clusters (mixing both approaches):

```rust
let cluster = BlockchainClusterBuilder::new()
    .with_node(
        // 3-space style node
        BlockchainNodeBuilder::new()
            .with_state(...)
            .with_brain(...)
            .with_executor(...)
            .build()
    )
    .with_node(
        // 6-dimension style node
        BlockchainNodeBuilder::new()
            .with_protocol(...)
            .with_storage(...)
            .with_cache(...)
            .with_transport(...)
            .with_external_interface(...)
            .with_timer(...)
            .build()
    )
    .build();
```

**Proves:**
- View equivalence (both APIs produce working nodes)
- Heterogeneous clusters work (different build styles coexist)
- Architecture is truly view-agnostic

### Validation Strategy

1. **Build same node both ways** - Identical behavior
2. **Mix in cluster** - 3-space nodes and 6-dimension nodes cooperate
3. **Performance equivalence** - No overhead from either builder style
4. **Test portability** - Same test harness works for both

**Success criteria**: Cluster with mixed builder styles reaches consensus, proving view equivalence is not just conceptual but operational.

---

## License

Licensed under the Apache License, Version 2.0.
