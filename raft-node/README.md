# raft-node

> Raft single-node implementation under the 3-6 architecture

`raft-node` is the core Raft crate in the independent `raft-*` protocol family. It provides the single-node `RaftNode<P>` realization of the 3-6 model, the pure `RaftProtocol<P>` consensus logic, swappable in-memory infrastructure, builders, observer support, and protocol-level tests.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md)
**Depended on by:** [raft-validation](../raft-validation/README.md), [raft-embassy](../raft-embassy/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate proves that the 3-6 architecture is not specific to the Ethereum-like / IBFT side of the workspace. Raft differs materially from IBFT in fault model, quorum logic, election behavior, replication model, and state evolution, yet still fits the same structural decomposition and the same `step()` primitive.

---

## Implemented Capabilities

### Core node model

- `RaftNode<P>` with incoming sources, state, executor, context builder, protocol, partitioner, state machine, and observer
- Pure `RaftProtocol<P>` handling timer, peer, and client messages through one decision surface
- Two-way action partitioning: mutations and outputs
- `RaftObserver` / `RaftActionKind` support for per-step visibility
- Builder-driven construction through `RaftNodeBuilder<P>` and variant-based component selection

### Consensus protocol

- Pre-vote and leader election
- Heartbeats and log replication through `AppendEntries`
- Majority-based commit advancement with the current-term rule
- Immediate step-down on higher-term messages
- Vote persistence before vote responses
- Log consistency checks and conflict truncation
- Snapshot installation and snapshot-aware log metadata helpers
- Client command append, query routing, and `NotLeader` redirects

### Swappable infrastructure implementations

- `InMemoryRaftStorage<P>`
- `InMemoryRaftCache`
- `InMemoryRaftTransport<P, S>`
- `InMemoryRaftTimer<S>`
- `InMemoryRaftExternalInterface<S>`
- `InMemoryRaftStateMachine`
- `NoOpRaftTransport<P>`
- `NoOpRaftObserver`
- `TypeBasedRaftPartitioner`
- `EagerRaftContextBuilder`

---

## Validation Surface

`raft-node` owns the Stage 1 protocol-level validation surface.

- 136 protocol and component tests in `tests/`
- Coverage across election, replication, snapshots, client behavior, role transitions, builders, storage, cache, timer, transport, observer logic, and property-based protocol checks

Stage 2 multi-node validation lives in [../raft-validation/README.md](../raft-validation/README.md). Stage 3 embedded validation lives in [../raft-embassy/README.md](../raft-embassy/README.md).

---

## Source Layout

```
src/
  raft_node.rs         # RaftNode<P> step-loop realization
  brain/               # Protocol alias and message handling surface
  builders/            # Builder APIs and variant-driven construction
  common_types/        # Raft messages, actions, client types, snapshots, roles
  context/             # Context builder and RaftContext<P>
  executor/            # Outgoing effect execution
  implementations/     # In-memory and no-op implementations, RaftProtocol<P>
  incoming/            # Polling of timer, transport, external interface
  observer.rs          # RaftObserver and RaftActionKind
  partitioner/         # Mutation/output partitioning
  state/               # Combined storage + cache view
  variants.rs          # Component variant enums
```

---

## Why This Crate Matters

`raft-node` is the strongest evidence that the EtheRAM architecture is genuinely general. If the same step-driven, pure-protocol, swappable-dimension node shape can support both IBFT and Raft, then the decomposition is doing real architectural work rather than merely mirroring one protocol family.
