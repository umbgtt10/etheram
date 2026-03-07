# EtheRAM

> **A research framework for blockchain node decomposition and abstraction**

EtheRAM explores how distributed system nodes can be decomposed, validated, and deployed across diverse environments — from in-memory simulation to `no_std` embedded targets running on ARM Cortex-M. Built on lessons from MetalRaft and Fluxion, it emphasizes explicit semantics, deterministic validation, and fully swappable components.

The primary artefact is **EtheRAM**: a minimal but real Ethereum-like node that validates the 3-6 architectural model under Byzantine consensus, embedded constraints, and Ethereum execution semantics.

---

## Project at a Glance

| Metric | Value |
|---|---|
| Crates | 10 (`core`, `embassy-core`, `etheram-node`, `etheram-validation`, `etheram-embassy`, `etheram-node-process`, `etheram-desktop`, `raft-node`, `raft-validation`, `raft-embassy`) |
| Production Rust files / LOC | ~190 / ~9 000 |
| Test files / LOC | ~75 / ~15 000 |
| Automated tests | 900+ across protocol, cluster, process, desktop, and QEMU validation |
| Consensus protocols | Istanbul BFT (PrePrepare → Prepare → Commit + View Change), Raft (pre-vote, election, log replication, snapshots) |
| Execution engines | `TinyEvmEngine`, `ValueTransferEngine`, `NoOpExecutionEngine` |
| Embedded target | ARM Cortex-M4 via QEMU, 5-node async cluster, two hardware configurations |
| Cryptographic schemes | `MockSignatureScheme`, `Ed25519SignatureScheme` (runtime-swappable) |

---

## Status

### Achieved

- **Full IBFT consensus** — three-phase commit, view change, quorum via `⌊2n/3⌋ + 1`, prepared certificate with cryptographic proof ([IBFT Roadmap](etheram-node/IBFT-ROADMAP.md))
- **Ethereum-like chain** — accounts, nonces, balances, gas metering, state roots, transaction receipts, block re-execution validation ([Chain Roadmap](etheram-node/CHAIN-ROADMAP.md))
- **TinyEVM** — subset EVM with opcode execution (`PUSH`, `ADD`, `MUL`, `SSTORE`, `SLOAD`, `RETURN`), per-opcode gas accounting, contract storage
- **748 automated tests** — protocol-level, cluster-level (Byzantine fault injection, deduplication, replay, malicious blocks, validator set updates), and QEMU end-to-end
- **Embedded port** — 5-node IBFT cluster on ARM Cortex-M4 (Embassy async, `no_std`, real Ed25519 signatures, semihosting storage, UDP transport)
- **Desktop multi-process cluster** — `etheram-node-process` + `etheram-desktop` provide a gRPC-connected native cluster with sled-backed state, gRPC client interface, WAL-backed restart recovery, and live partition control
- **Total component swappability** — storage, cache, transport, timer, external interface, context builder, partitioner, execution engine, signature scheme, observer — all swappable at construction time
- **Ed25519 cryptographic signatures** — real signing/verification integrated into consensus flow; `PreparedCertificate` carries quorum proof
- **WAL crash-recovery** — `ConsensusWal` serialization/deserialization with restart recovery
- **Raft consensus** \u2014 second protocol family (`raft-node`, `raft-validation`, `raft-embassy`) proving decomposition generality across CrashFault+CFT consensus ([Raft Roadmap](raft-node/RAFT-ROADMAP.md))

### Planned

- Physical hardware deployment (STM32 / RP2040)
- Merkle Patricia Trie state root
- JSON-RPC external interface
- BLS round-change certificate aggregation

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
embassy-core/               Shared no_std Embassy infrastructure for both protocol families
etheram-node/               Ethereum-like node (types + concrete implementations)
etheram-validation/         Cluster/integration tests (multi-node, std)
etheram-embassy/            no_std + Embassy embedded port (ARM Cortex-M4)
etheram-node-process/       One-node desktop/runtime process with gRPC transport, gRPC external interface, sled, and WAL
etheram-desktop/            Native launcher + dashboard for the desktop multi-process cluster
raft-node/                  Raft node (types + concrete implementations)
raft-validation/            Raft cluster/integration tests (multi-node, std)
raft-embassy/               no_std + Embassy embedded port for Raft
docs/                       Architecture docs and ADRs
scripts/                    Workspace quality gates and demo runners
```

### Crate Dependency Graph

```
               ┌── etheram-node ←── etheram-validation
               │              ├── etheram-embassy
               │              └── etheram-node-process ←── etheram-desktop
core ──────────┤
               ├── raft-node ←──── raft-validation
               │          └──── raft-embassy
               └── embassy-core ←─ etheram-embassy, raft-embassy
```

Dependencies are strictly one-way by protocol family. Node crates never depend on validation/embassy crates. `core`, `embassy-core`, `etheram-node`, `raft-node`, `etheram-embassy`, and `raft-embassy` are `no_std`-compatible.

### Per-Crate Documentation

| Crate | README | Purpose |
|---|---|---|
| `core` | [core/README.md](core/README.md) | Foundational traits (`ConsensusProtocol`, `Node`, dimension I/O) |
| `embassy-core` | [embassy-core/README.md](embassy-core/README.md) | Shared Embassy infrastructure (network bus, channels, timer hubs, client facade macros) |
| `etheram-node` | [etheram-node/README.md](etheram-node/README.md) | Ethereum-like node types, concrete implementations, IBFT protocol, builders |
| `etheram-validation` | [etheram-validation/README.md](etheram-validation/README.md) | Multi-node cluster harness and integration tests |
| `etheram-embassy` | [etheram-embassy/README.md](etheram-embassy/README.md) | `no_std` + Embassy ARM port with QEMU validation |
| `etheram-node-process` | [etheram-node-process/README.md](etheram-node-process/README.md) | One-node desktop runtime with gRPC networking, sled persistence, and WAL recovery |
| `etheram-desktop` | [etheram-desktop/README.md](etheram-desktop/README.md) | Native launcher + dashboard for the multi-process desktop cluster |
| `raft-node` | [raft-node/README.md](raft-node/README.md) | Raft node types, concrete implementations, protocol logic |
| `raft-validation` | [raft-validation/README.md](raft-validation/README.md) | Multi-node Raft cluster harness and integration tests |
| `raft-embassy` | [raft-embassy/README.md](raft-embassy/README.md) | `no_std` + Embassy ARM port for Raft |

### Roadmaps

| Document | Scope |
|---|---|
| [IBFT-ROADMAP.md](etheram-node/IBFT-ROADMAP.md) | IBFT consensus protocol features (supported + planned) |
| [CHAIN-ROADMAP.md](etheram-node/CHAIN-ROADMAP.md) | Ethereum-like chain features (supported + planned) |
| [RAFT-ROADMAP.md](raft-node/RAFT-ROADMAP.md) | Raft consensus implementation plan (second protocol family) |

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
| 1 | `etheram-node/tests/` + `raft-node/tests/` | Protocol-level unit tests (pure logic, isolated components) |
| 2 | `etheram-validation/tests/` | Cluster-level integration tests (multi-node, Byzantine fault injection) |
| 3 | `etheram-embassy/` + `raft-embassy/` (QEMU) | Embedded end-to-end (ARM Cortex-M4, `no_std`, async, real crypto) |

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
