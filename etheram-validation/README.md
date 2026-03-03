# etheram-validation

> Multi-node cluster harness and integration tests

`etheram-validation` provides the test infrastructure for Stage 2 validation: orchestrating multiple `EtheramNode` instances in a shared-memory cluster, driving consensus rounds, injecting faults, and asserting distributed correctness. This is the only `std` crate in the EtheRAM ecosystem — it uses `Arc`, `Mutex`, and standard library collections for cluster orchestration.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [etheram](../etheram/README.md), [etheram-variants](../etheram-variants/README.md)

---

## Contents

### IbftCluster (`ibft_cluster.rs`)

Orchestrates a configurable number of `EtheramNode` instances wired with `InMemoryTransport`, `InMemoryStorage`, `InMemoryTimer`, and `InMemoryExternalInterface`. Provides methods to drive the cluster deterministically:

| Method | Purpose |
|---|---|
| `new(node_count)` | Create a cluster with `n` nodes and shared transport state |
| `fire_timer(node_index, event)` | Inject a timer event into a specific node |
| `submit_request(node_index, request)` | Submit a client request to a specific node |
| `drain_responses(node_index)` | Collect all client responses from a node |
| `step_all()` | Step all nodes once; returns total work done |
| `step_until_quiet()` | Step all nodes until no node has pending work |
| `push_transport_message(node_index, message)` | Inject a raw transport message |
| `query_height(node_index)` | Query a node's committed height |
| `query_account(node_index, address)` | Query an account balance/nonce |
| `with_genesis_account(address, account)` | Seed genesis accounts across all nodes |
| `with_execution_engine(variant)` | Set the execution engine for all nodes |
| `with_validator_set_update(update)` | Schedule a validator set transition |

### IbftTestNode (`ibft_test_node.rs`)

Single-node test wrapper for protocol-level tests that need an `EtheramNode` without a full cluster. Used in `etheram_node_tests.rs`.

### StdSharedState (`std_shared_state.rs`)

`Arc<Mutex<InMemoryTransportState>>` wrapper that implements the shared transport state for in-process cluster communication.

---

## Tests

145 tests across 16 test files in `tests/`:

### Single-Node Integration (`etheram_node_tests.rs`)

3 tests validating `EtheramNode` construction and step behavior with concrete implementations.

### Cluster Tests (15 files)

| File | Count | Scope |
|---|---|---|
| `ibft_cluster_basics_tests.rs` | — | Basic cluster construction, initial height, proposer identity |
| `ibft_cluster_round_progression_tests.rs` | — | Multi-round consensus, height advancement, proposer rotation |
| `ibft_cluster_view_change_tests.rs` | — | TimeoutRound → ViewChange → NewView → resume |
| `ibft_cluster_client_tests.rs` | — | Client request submission and response collection |
| `ibft_cluster_transaction_tests.rs` | — | Balance transfers, nonce validation, gas limit enforcement |
| `ibft_cluster_byzantine_tests.rs` | — | Byzantine fault injection (conflicting blocks, forged certs) |
| `ibft_cluster_dedup_tests.rs` | — | Duplicate message filtering |
| `ibft_cluster_replay_tests.rs` | — | Message replay prevention |
| `ibft_cluster_persistence_tests.rs` | — | WAL save/restore after restart |
| `ibft_cluster_malicious_block_tests.rs` | — | Malicious block rejection, sender quarantine |
| `ibft_cluster_message_validation_tests.rs` | — | Invalid message filtering (wrong height, wrong proposer, non-validator) |
| `ibft_cluster_gas_tests.rs` | — | Gas metering: intrinsic gas, out-of-gas, mixed success/failure |
| `ibft_cluster_execution_engine_tests.rs` | — | Engine swap validation (TinyEvm, ValueTransfer, NoOp) |
| `ibft_cluster_validator_updates_tests.rs` | — | Height-gated validator set transitions |
| `ibft_cluster_reexecution_tests.rs` | — | Block re-execution validation, multi-height commitment consistency |

All tests are deterministic and fully in-memory. No real I/O, no timers, no threads — the test harness controls every input.

---

## Source Layout

```
src/
  lib.rs                    pub mod declarations
  ibft_cluster.rs           IbftCluster builder and orchestrator
  ibft_test_node.rs         IbftTestNode single-node wrapper
  std_shared_state.rs       Arc<Mutex<...>> shared transport state
tests/
  all_tests.rs              Single integration test entry point
  etheram_node_tests.rs     Single-node integration tests
  ibft_cluster_*_tests.rs   15 cluster test files (145 tests total)
  common/                   Shared test helpers
```

---

## Usage Pattern

```rust
let mut cluster = IbftCluster::new(4)
    .with_execution_engine(ExecutionEngineVariant::TinyEvm)
    .with_genesis_account(sender, Account::new(1000))
    .build();

// Drive a full consensus round
cluster.fire_timer(0, TimerEvent::ProposeBlock);
cluster.step_until_quiet();

// All 4 nodes committed height 1
for i in 0..4 {
    assert_eq!(cluster.query_height(i), 1);
}
```
