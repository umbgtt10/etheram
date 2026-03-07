# etheram-node-process

> One-node desktop runtime process for the Direction H multi-process cluster

`etheram-node-process` runs exactly one EtheRAM node as a native `std` process. It wires the core `EtheramNode` to desktop-oriented infrastructure: gRPC peer transport, gRPC client `ExternalInterface`, sled-backed storage, sync handling, partition-control commands, and file-backed WAL recovery.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [etheram-node](../etheram-node/README.md)
**Used by:** [etheram-desktop](../etheram-desktop/README.md)

---

## Purpose

This crate is the per-node runtime for Direction H. Each OS process hosts one validator and is driven by a `cluster.toml` entry plus a node id.

It proves that the 3-6 architecture survives real process boundaries while preserving the same single-node core logic used in in-memory and embedded validation.

---

## Runtime Responsibilities

- Parse fleet and per-node configuration from `cluster.toml`
- Build one `EtheramNode<IbftMessage>` with desktop infrastructure
- Expose peer transport over gRPC on `transport_addr`
- Expose client RPCs over a separate gRPC `ExternalInterface` on `client_addr`
- Persist blocks, accounts, receipts, and contract storage in sled under `db_path`
- Persist and restore `ConsensusWal` under `db_path/consensus.wal`
- Accept live partition-control commands over stdin: `partition`, `heal`, `clear`, `shutdown`

---

## Main Components

### `NodeRuntime`

`NodeRuntime` in `src/etheram_node.rs` is the std wrapper around the core step loop. It owns:

- gRPC peer transport
- gRPC external-interface server
- sled storage
- file-backed WAL
- sync handler
- timer scheduler
- partition table

The runtime either runs forever or for a finite step budget, depending on CLI arguments.

### gRPC peer transport

Peer-to-peer IBFT and sync messages flow through the transport infrastructure under `src/infra/transport/`.

- Peer-facing gRPC is separate from client-facing gRPC
- Sync traffic uses dedicated message/bus paths
- Partition simulation is enforced in-process before transport delivery

### gRPC external interface

The client API lives under `src/infra/external_interface/` and exposes unary RPCs for:

- `SubmitTransaction`
- `GetBalance`
- `GetHeight`
- `GetBlock`

This is the H5 client-facing desktop boundary.

### Storage and WAL

Storage under `src/infra/storage/` uses sled for durable desktop persistence.

WAL under `src/infra/wal/` persists `ConsensusWal` to a file so a restarted process reloads:

- current round
- pending block
- prepared certificate
- other protocol replay/persistence state

This is the H6 crash-recovery path.

---

## Configuration

The process is launched as:

```text
etheram-node-process <cluster.toml> <node-id> [step-limit]
```

- `cluster.toml` selects addresses, db path, and validator set
- `node-id` chooses the single `[[node]]` entry to run
- `step-limit = 0` means run forever

---

## Tests

The integration suite covers:

- config parsing and validation
- startup and CLI behavior
- gRPC external interface component tests
- process-level external interface tests
- sled persistence and restart behavior
- file-backed WAL round-trip and invalid data handling
- H6 restart recovery from seeded locked-block WAL

The test entry point is `tests/all_tests.rs`.
