# raft-embassy

> `no_std` + Embassy embedded port for the Raft protocol family

`raft-embassy` is the embedded Stage 3 realization of the Raft side of the workspace. It validates Raft end-to-end on ARM Cortex-M4 in QEMU across two mandatory configurations: all-in-memory and UDP + semihosting.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [embassy-core](../embassy-core/README.md), [raft-node](../raft-node/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate answers the embedded-side question for Raft: does the same Raft node architecture remain valid under `no_std`, Embassy async scheduling, and ARM resource constraints?

It is not a production firmware image. It is an architecture and correctness proof under an embedded runtime.

---

## Implemented Embedded Surface

### Two mandatory configurations

Both configurations are feature-gated and must remain viable:

| Configuration | Transport | Storage | External Interface |
|---|---|---|---|
| All-in-memory | channel transport | in-memory storage | channel external interface |
| Real | UDP transport | semihosting storage | UDP external interface |

### Runtime model

- `#![no_std]` + `#![no_main]`
- Embassy async task-per-node execution
- shared `CancellationToken` for coordinated shutdown
- embedded logging, heap initialization, and time-driver setup
- node creation through `configurations::init()` and per-node spawning helpers

### QEMU scenario currently proven

`src/main.rs` drives a five-act Raft demonstration:

- Act 0: election
- Act 1: replication
- Act 2: read-after-write query
- Act 3: re-election after timeout pressure
- Act 4: continued replication under the new leader

This proves leader election, client submission, replicated commit, query-after-commit behavior, term advancement, and post-election continued liveness in the embedded environment.

### Embedded infrastructure axes

- channel vs UDP transport
- in-memory vs semihosting storage
- channel vs UDP external interface
- Embassy time-driver and shared-state wiring
- client facade and observer integration for the embedded runtime

---

## Source Layout

```
src/
  main.rs                # five-act QEMU scenario
  config.rs
  configurations/        # in-memory and real embedded wiring
  infra/                 # transport, storage, external interface variants
  raft_client.rs         # embedded client facade for scenario driving
  raft_observer.rs       # embedded observer surface
  spawned_node.rs        # per-node runtime wiring
  logging.rs
  time_driver.rs
  embassy_shared_state.rs
  heap.rs
```

---

## Why This Crate Matters

`raft-embassy` closes the generality argument at the embedded layer. It shows that the same `no_std` execution environment used for the Etheram / IBFT side can host a materially different protocol family without changing the architectural model.
