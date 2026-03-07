# raft-validation

> Multi-node Raft cluster harness and Stage 2 validation

`raft-validation` provides the deterministic multi-node validation layer for the Raft protocol family. It supplies the `RaftCluster` harness plus integration tests that exercise election, replication, fault tolerance, snapshot behavior, state machine application, and client interaction across a full cluster rather than isolated protocol handlers.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [raft-node](../raft-node/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate is the Stage 2 proof that the Raft implementation behaves correctly once multiple nodes interact through the full step loop. It fills the gap between single-node protocol correctness and embedded deployment by validating cluster-level behavior in a deterministic shared-memory environment.

---

## Implemented Validation Surface

- `RaftCluster` harness exposed from `src/raft_cluster.rs`
- Cluster stepping helpers for single-node and whole-cluster progression
- Timer injection for deterministic election and heartbeat control
- Message injection for targeted fault and snapshot scenarios
- Client command and query submission paths
- Response draining and node-state inspection helpers

The current suite covers 60 cluster-level tests across:

- election and leader stability
- replication and commit advancement
- minority and majority fault-tolerance cases
- snapshot installation and recovery behavior
- state machine application consistency
- client redirects, commands, and read-after-write flows

---

## Relationship To Other Crates

- [../raft-node/README.md](../raft-node/README.md) owns the single-node protocol and component layer
- this crate owns the cluster-level distributed-correctness layer
- [../raft-embassy/README.md](../raft-embassy/README.md) owns the embedded `no_std` deployment layer

Together they form the Raft-side equivalent of the Etheram Stage 1 / Stage 2 / Stage 3 validation story.

---

## Source Layout

```
src/
  lib.rs           pub mod declarations
  raft_cluster.rs  RaftCluster harness and cluster orchestration
tests/
  all_tests.rs                 single integration test entry point
  election_tests.rs            leader election and term progression
  replication_tests.rs         append/commit behavior across the cluster
  fault_tolerance_tests.rs     crash and partition scenarios
  snapshot_tests.rs            snapshot install and recovery behavior
  state_machine_tests.rs       applied-state consistency
  client_tests.rs              client command, query, and redirect flows
  cluster_api_tests.rs         harness-level stepping and injection behavior
  common/                      shared test helpers
```

---

## Why This Crate Matters

`raft-validation` is the distributed-correctness proof for the Raft family. It demonstrates that the protocol is not only locally coherent in Stage 1, but also stable under cluster interaction, retries, elections, faults, and snapshot flows.
