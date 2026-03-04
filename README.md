# EtheRAM

> **A research framework for blockchain node decomposition and abstraction**

EtheRAM explores how distributed system nodes can be decomposed, validated, and deployed across diverse environments — from in-memory simulation to `no_std` embedded targets running on ARM Cortex-M. Built on lessons from MetalRaft and Fluxion, it emphasizes explicit semantics, deterministic validation, and fully swappable components.

The primary artefact is **EtheRAM**: a minimal but real Ethereum-like node that validates the 3-6 architectural model under Byzantine consensus, embedded constraints, and Ethereum execution semantics.

---

## Project at a Glance

| Metric | Value |
|---|---|
| Crates | 5 (`core`, `etheram`, `etheram-variants`, `etheram-validation`, `etheram-embassy`) |
| Production Rust files / LOC | ~190 / ~9 000 |
| Test files / LOC | ~75 / ~15 000 |
| Automated tests | 557 (21 + 391 + 145) |
| Consensus protocol | Istanbul BFT (PrePrepare → Prepare → Commit + View Change) |
| Execution engines | `TinyEvmEngine`, `ValueTransferEngine`, `NoOpExecutionEngine` |
| Embedded target | ARM Cortex-M4 via QEMU, 5-node async cluster, two hardware configurations |
| Cryptographic schemes | `MockSignatureScheme`, `Ed25519SignatureScheme` (runtime-swappable) |

---

## Status

### Achieved

- **Full IBFT consensus** — three-phase commit, view change, quorum via `⌊2n/3⌋ + 1`, prepared certificate with cryptographic proof ([IBFT Roadmap](etheram/IBFT-ROADMAP.md))
- **Ethereum-like chain** — accounts, nonces, balances, gas metering, state roots, transaction receipts, block re-execution validation ([Chain Roadmap](etheram/CHAIN-ROADMAP.md))
- **TinyEVM** — subset EVM with opcode execution (`PUSH`, `ADD`, `MUL`, `SSTORE`, `SLOAD`, `RETURN`), per-opcode gas accounting, contract storage
- **557 automated tests** — protocol-level, cluster-level (Byzantine fault injection, deduplication, replay, malicious blocks, validator set updates), and QEMU end-to-end
- **Embedded port** — 5-node IBFT cluster on ARM Cortex-M4 (Embassy async, `no_std`, real Ed25519 signatures, semihosting storage, UDP transport)
- **Total component swappability** — storage, cache, transport, timer, external interface, context builder, partitioner, execution engine, signature scheme, observer — all swappable at construction time
- **Ed25519 cryptographic signatures** — real signing/verification integrated into consensus flow; `PreparedCertificate` carries quorum proof
- **WAL crash-recovery** — `ConsensusWal` serialization/deserialization with restart recovery

### Planned

- **Raft consensus** — second protocol family (`raft-node`, `raft-variants`, `raft-validation`, `raft-embassy`) to prove decomposition generality ([Raft Roadmap](etheram/RAFT-ROADMAP.md))
- Physical hardware deployment (STM32 / RP2040)
- Property-based testing (`proptest`)
- Merkle Patricia Trie state root
- Formal specification (TLA+)

---

## The 3-6 Architectural Model

Every node is decomposed along two orthogonal axes:

### 6 Dimensions (structural — what components exist)

| # | Dimension | Role |
|---|---|---|
| 1 | **Protocol** | Consensus logic (pure, stateless) |
| 2 | **Storage** | Persistent state |
| 3 | **Cache** | Volatile working state |
| 4 | **Transport** | Peer-to-peer communication |
| 5 | **ExternalInterface** | Client request/response |
| 6 | **Timer** | Time-based event scheduling |

### 3 Spaces (functional — what roles components play)

| # | Space | Responsibility |
|---|---|---|
| 1 | **Brain Space** | Build context → handle message → produce actions |
| 2 | **Scheduler Space** | Poll dimensions → select next event → dispatch |
| 3 | **Dimension Space** | Data dimensions (Storage, Cache) and I/O dimensions (Transport, ExternalInterface, Timer) |

**Key insight:** Every dimension can be swapped independently. Protocol logic is a pure function — no I/O, no side effects. The `Partitioner` separates state mutations from output effects and block executions. This enables exhaustive testing through component substitution, not scenario scripting.

---

## Workspace Structure

```
core/                       Shared abstractions (PeerId, dimension traits, ConsensusProtocol)
etheram/                    Core node implementation (no_std, single-node logic)
etheram-variants/           Concrete implementations + builder API
etheram-validation/         Cluster/integration tests (multi-node, std)
etheram-embassy/            no_std + Embassy embedded port (ARM Cortex-M4)
docs/                       Architecture docs and ADRs
scripts/                    CI gate script (test.ps1)
```

### Crate Dependency Graph

```
core ←── etheram ←── etheram-variants ←── etheram-validation
                                     ←── etheram-embassy
```

Dependencies are strictly one-way. `etheram` never depends on `etheram-variants`. All crates except `etheram-validation` are `no_std`-compatible.

### Per-Crate Documentation

| Crate | README | Purpose |
|---|---|---|
| `core` | [core/README.md](core/README.md) | Foundational traits (`ConsensusProtocol`, `Node`, dimension I/O) |
| `etheram` | [etheram/README.md](etheram/README.md) | Core node types, step loop, execution engine trait, observer |
| `etheram-variants` | [etheram-variants/README.md](etheram-variants/README.md) | Concrete implementations, IBFT protocol, builders |
| `etheram-validation` | [etheram-validation/README.md](etheram-validation/README.md) | Multi-node cluster harness and integration tests |
| `etheram-embassy` | [etheram-embassy/README.md](etheram-embassy/README.md) | `no_std` + Embassy ARM port with QEMU validation |

### Roadmaps

| Document | Scope |
|---|---|
| [IBFT-ROADMAP.md](etheram/IBFT-ROADMAP.md) | IBFT consensus protocol features (supported + planned) |
| [CHAIN-ROADMAP.md](etheram/CHAIN-ROADMAP.md) | Ethereum-like chain features (supported + planned) |
| [RAFT-ROADMAP.md](etheram/RAFT-ROADMAP.md) | Raft consensus implementation plan (second protocol family) |

---

## Core Execution Model

All nodes share one execution primitive — **`step()`**:

```rust
fn step(&mut self) -> bool {
    if let Some((source, message)) = self.incoming.poll() {
        let context = self.context_builder.build(&self.state, self.peer_id, &source, &message);
        let actions = self.brain.handle_message(&source, &message, &context);
        let (mutations, outputs, executions) = self.partitioner.partition(&actions);
        self.state.apply_mutations(&mutations);
        self.executor.execute_outputs(&outputs);
        // executions trigger ExecutionEngine for transaction processing
        return true;
    }
    false
}
```

**Properties:** non-blocking, deterministic, runtime-agnostic. The same `step()` drives sequential polling, async select (Embassy), and multi-node cluster harnesses.

---

## Documentation

### Architecture Decision Records

- **[ADR-001: Six-Dimension Node Decomposition](docs/ADR/001-six-dimension-node-decomposition.md)**
- **[ADR-002: step() as Single Execution Primitive](docs/ADR/002-step-as-single-execution-primitive.md)**

### Design Documentation

- **[Architecture Overview](docs/ARCHITECTURE.md)**

---

## Three-Stage Validation

Every feature is validated across three stages:

| Stage | Where | What |
|---|---|---|
| 1 | `etheram-variants/tests/` | Protocol-level unit tests (pure logic, isolated components) |
| 2 | `etheram-validation/tests/` | Cluster-level integration tests (multi-node, Byzantine fault injection) |
| 3 | `etheram-embassy/` (QEMU) | Embedded end-to-end (ARM Cortex-M4, `no_std`, async, real crypto) |

The CI gate script (`scripts/test.ps1`) runs all three stages plus `cargo fmt` and `no_std` compatibility checks.

---

## Future Directions

### Beyond Blockchain

The 6-dimension decomposition generalizes to any distributed agent:
- UAV swarms (Protocol = coordination algorithm, Transport = radio mesh)
- Autonomous vehicles (Storage = map state, Timer = waypoint scheduling)
- Distributed control systems (ExternalInterface = ground control, Cache = sensor data)

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- **MetalRaft** — Deterministic Raft experimentation
- **Fluxion** — Runtime-agnostic async streams
- **MIT 6.5840 Distributed Systems** — Methodological inspiration
