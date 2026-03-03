# BareChain

> **A research framework for blockchain node decomposition and abstraction**

BareChain explores how distributed system nodes can be abstracted, validated, and deployed across diverse environments—from in-memory simulation to no-std embedded targets. Built on lessons from MetalRaft and Fluxion, it emphasizes explicit semantics, deterministic validation, and pluggable components.

## Status Overview

### ✅ Achieved

- **6-dimension node model** - Complete decomposition proven ([ADR-001](docs/adr/001-six-dimension-node-decomposition.md))
- **Sequential execution** - TinyChain with polling-based `step()` ([TinyChain README](examples/tinychain/README.md))
- **Async execution** - Embassy async with event-driven architecture ([Embassy README](examples/embassy-async/README.md))
- **Adversarial testing** - 20 deterministic tests with full control

### 🔄 Starting

- **EtheRAM** - Byzantine Ethereum on embedded hardware
  - Phase 0: Project skeleton with minimal abstractions
  - Next: Simplest BFT consensus algorithm

([EtheRAM README](etheram/README.md))
([EtheRAM ROADMAP](etheram/ROADMAP.md))

### 📋 Planned

- BFT consensus with full Byzantine test harness
- Ethereum semantics (accounts, state roots, gas, merkle trees)
- 6-dimension + 3-space formalization
- Dual builder APIs (structural vs functional views)
- Physical hardware deployment (RP2040/STM32)
- Cryptographic algorithm swap validation

---

## Overview

### The 6-Dimension Model

BareChain decomposes blockchain nodes into **six orthogonal dimensions**:

1. **Protocol** — Consensus algorithm (stateless, pure functions)
2. **Storage** — Persistent state (crash recovery)
3. **Cache** — Volatile working state (algorithm-specific)
4. **Transport** — Peer-to-peer communication
5. **ExternalInterface** — Client request/response
6. **Timer** — Time-based event scheduling

**Key insight:** Each dimension can be swapped independently. Protocol logic remains pure, infrastructure adapts.

### Core Principles

- **Maximal abstraction** - Define WHAT, not HOW
- **Protocol-specific types** - Infrastructure uses protocol types, not generic events
- **Validation first** - Deterministic, adversarial testing without real I/O
- **Execution model independence** - Same logic works sequential, async, embedded

**[Complete architectural analysis](docs/ARCHITECTURE.md)**

---

## Examples

### TinyChain: Sequential Polling

**[TinyChain](examples/tinychain/)** validates the 6-dimension model through minimal blockchain implementation.

**Objectives:**
- ✅ Six-dimension decomposition
- ✅ Adversarial test harness (easy setup, deterministic, repeatable)
- ✅ Lazy sequential execution (`step()` primitive)
- ✅ Static instantiation (no dynamic dispatch)

**Features:**
- Simple leader rotation consensus
- 20 deterministic validation tests
- Zero mutable getters
- In-memory simulation

```bash
cd examples/tinychain
cargo test  # All 20 tests pass
```

**[→ TinyChain README](examples/tinychain/README.md)**

---

### Embassy-Async: Event-Driven Execution

**[Embassy-Async](examples/embassy-async/)** proves the same architecture works with async runtime on embedded hardware.

**Objectives:**
- ✅ Blockchain on no-std + Embassy
- ✅ Dynamic dimension injection (trait objects)
- ✅ Async select event multiplexing
- ✅ QEMU ARM Cortex-M simulation

**Features:**
- 3-node consensus cluster
- Event-driven with `select4`
- In-memory channels and TCP transport
- Embassy async executor

```bash
cd examples/embassy-async
cargo run  # QEMU simulation
```

**[→ Embassy-Async README](examples/embassy-async/README.md)**

---

### EtheRAM: Byzantine Ethereum (In Development)

**[EtheRAM](etheram/)** validates the architecture under production constraints: Byzantine consensus, Ethereum semantics, and embedded deployment.

**Objectives:**
- 🔄 Simplest BFT consensus algorithm
- 🔄 Ethereum state machine (accounts, gas, merkle trees)
- 🔄 6-dimension + 3-space formalization
- 🔄 Dual builder APIs (structural vs functional)
- 🔄 Simulated embedded hardware (5-node cluster on QEMU)
- 🔄 Crypto algorithm swap validation

**Current Phase:** Project skeleton with minimal abstractions

**[→ EtheRAM README](etheram/README.md)**

---

## Key Discoveries

### 1. Dual Architectural Views

The architecture can be viewed through two lenses:

**6-Dimension View (Structural)** - What components exist
- Protocol, Storage, Cache, Transport, ExternalInterface, Timer

**3-Space View (Functional)** - What roles components play
- Brain Space (build_context, handle_message, execute_actions)
- Scheduler Space (event selection strategy)
- Dimension Space (I/O vs data dimensions)

**EtheRAM will formalize the relationship** between these views and prove equivalence through dual builder APIs.

### 2. Execution Model Independence

The same `step()` primitive supports:
- Sequential polling (TinyChain)
- Async select (Embassy)
- Any other execution strategy

**Scheduler is orthogonal to protocol logic** - proven by two working implementations.

---

## Documentation

### Architecture Decision Records

- **[ADR-001: Six-Dimension Node Decomposition](docs/adr/001-six-dimension-node-decomposition.md)** - Rationale for the 6-dimension model
- **[ADR-002: step() as Single Execution Primitive](docs/adr/002-step-as-single-execution-primitive.md)** - Why one method suffices

### Implementation Guides

- **[TinyChain README](examples/tinychain/README.md)** - Sequential polling implementation
- **[Embassy-Async README](examples/embassy-async/README.md)** - Async event-driven implementation
- **[EtheRAM README](etheram/README.md)** - Byzantine Ethereum roadmap

### Design Documentation

- **[Architecture Overview](docs/ARCHITECTURE.md)** - Complete architectural analysis

---

## Future Directions

### Beyond Blockchain

The Node abstraction (of which blockchain nodes are a specific instantiation) generalizes to:
- UAV swarms
- Autonomous vehicles
- Distributed control systems
- Robot coordination

Each agent executes a coordination protocol with the same six dimensions:
- Protocol (swarm coordination algorithm)
- Storage (mission state, waypoints)
- Cache (neighbor positions, local sensor data)
- Transport (radio, mesh network)
- ExternalInterface (ground control)
- Timer (waypoint scheduling, heartbeats)

---

## Philosophy

BareChain is not a product. It's a **research-grade engineering exploration** focused on:
- Abstraction
- Validation
- Correctness
- Deployability

The goal: separate **what must be correct** from **what may vary**.

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- **MetalRaft** — Deterministic Raft experimentation
- **Fluxion** — Runtime-agnostic async streams
- **MIT 6.5840 Distributed Systems** — Methodological inspiration
