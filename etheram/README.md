# etheram

> Core node implementation — types, traits, step loop, and execution engine interface

`etheram` defines the single-node architecture for an Ethereum-like blockchain node. It contains the `EtheramNode` struct and step loop, all common types (blocks, accounts, transactions, state roots, receipts), the execution engine trait, the observer trait, and the adapter layer that bridges core dimension traits to Ethereum-specific types.

This crate is `#![no_std]` and uses `alloc` for heap types. It contains **no concrete implementations** — those live in [etheram-variants](../etheram-variants/README.md).

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md)
**Depended on by:** [etheram-variants](../etheram-variants/README.md), and transitively by [etheram-validation](../etheram-validation/README.md) and [etheram-embassy](../etheram-embassy/README.md)

---

## Roadmaps

| Document | Scope |
|---|---|
| [IBFT-ROADMAP.md](IBFT-ROADMAP.md) | IBFT consensus protocol features (supported + planned) |
| [CHAIN-ROADMAP.md](CHAIN-ROADMAP.md) | Ethereum-like chain features (supported + planned) |
| [RAFT-ROADMAP.md](RAFT-ROADMAP.md) | Raft consensus implementation plan (second protocol family) |

---

## EtheramNode

The central struct that wires infrastructure (dimensions) and decision (brain, context, partitioner) together:

```rust
pub struct EtheramNode<M: Clone + 'static> {
    peer_id: PeerId,
    incoming: IncomingSources<M>,       // polls Timer, ExternalInterface, Transport
    state: EtheramState,                // wraps Storage + Cache
    executor: EtheramExecutor<M>,       // executes output actions
    context_builder: Box<dyn ContextBuilder<M>>,
    brain: BoxedProtocol<M>,
    partitioner: Box<dyn Partitioner<M>>,
    execution_engine: BoxedExecutionEngine,
    observer: Box<dyn Observer>,
}
```

Every component is injected via trait objects (`Box<dyn Trait>`) and is swappable at construction time.

### The Step Loop

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

Protocol logic is pure: `handle_message` takes immutable context, returns declarative actions. No I/O occurs inside the protocol.

---

## Contents

### Common Types (`common_types/`)

| Type | File | Purpose |
|---|---|---|
| `Account` | `account.rs` | Balance + nonce |
| `Block` | `block.rs` | Height, proposer, transactions, state_root, post_state_root, receipts_root; `compute_hash()` |
| `Transaction` | `transaction.rs` | From, to, value, gas_limit, nonce, data |
| `Address`, `Hash` | `types.rs` | `[u8; 20]`, `[u8; 32]` type aliases |

State root computation: deterministic XOR-mix hash over sorted `BTreeMap<Address, Account>`, implemented in `state_root.rs`.

### Dimension Adapters (`common_types/`)

Adapter traits that specialize core dimension traits for Ethereum-specific types:

| Adapter | Core Trait | Specialization |
|---|---|---|
| `StorageAdapter` | `Storage` | `StorageQuery` / `StorageMutation` |
| `CacheAdapter` | `Cache` | `CacheQuery` / `CacheMutation` |
| `TimerInputAdapter` | `TimerInput` | `TimerEvent` |
| `TimerOutputAdapter` | `TimerOutput` | `TimerEvent`, `TimerDelay` |
| `TransportIncomingAdapter` | `TransportIncoming` | `IbftMessage` (generic `M`) |
| `TransportOutgoingAdapter` | `TransportOutgoing` | `IbftMessage` (generic `M`) |
| `ExternalInterfaceIncomingAdapter` | `ExternalInterfaceIncoming` | `ClientRequest` |
| `ExternalInterfaceOutgoingAdapter` | `ExternalInterfaceOutgoing` | `ClientResponse` |

### Execution Engine (`execution/`)

| Type | File | Purpose |
|---|---|---|
| `ExecutionEngine` trait | `execution_engine.rs` | `execute(block, accounts, contract_storage) → Vec<ExecutionResult>` |
| `BoxedExecutionEngine` | `execution_engine.rs` | `Box<dyn ExecutionEngine>` alias |
| `ExecutionResult` | `execution_result.rs` | Per-transaction outcome (mutations + gas + status) |
| `TransactionResult` | `transaction_result.rs` | Success / OutOfGas |
| `TransactionReceipt` | `transaction_receipt.rs` | Status + gas_used + cumulative_gas_used |
| `compute_receipts_root()` | `receipts_root.rs` | Deterministic hash over receipt list |
| `compute_block_commitments()` | `block_commitments.rs` | Centralized commitment computation (post_state_root + receipts_root) |

The `ExecutionEngine` trait contract: immutable input, declarative `ExecutionResult`, no I/O. Concrete engines (`TinyEvmEngine`, `ValueTransferEngine`, `NoOpExecutionEngine`) are in `etheram-variants`.

### Brain (`brain/`)

Contains the `Protocol` type alias (`BoxedProtocol = Box<dyn ConsensusProtocol<...>>`) that adapts the core `ConsensusProtocol` trait to Ethereum-specific action/context types.

### Context (`context/`)

| Type | File | Purpose |
|---|---|---|
| `ContextBuilder` trait | `context_builder.rs` | `build(state, peer_id, source, message) → ContextDto` |
| `ContextDto` | `context_dto.rs` | Snapshot of current height, accounts, pending txs, state root, contract storage |

### Partitioner (`partitioner/`)

| Type | File | Purpose |
|---|---|---|
| `Partitioner` trait | `partition.rs` | `partition(actions) → (mutations, outputs, executions)` |

Separates state mutations from output effects and block executions. Enforces side-effect isolation.

### State (`state/`)

| Type | File | Purpose |
|---|---|---|
| `EtheramState` | `etheram_state.rs` | Wraps `Box<dyn StorageAdapter>` + `Box<dyn CacheAdapter>` |

Provides `apply_mutations()` and query methods (`query_height`, `query_account`, `query_state_root`, `query_pending_transactions`, `query_contract_storage`).

### Incoming (`incoming/`)

| Type | File | Purpose |
|---|---|---|
| `IncomingSources` | `incoming_sources.rs` | Polls Timer → ExternalInterface → Transport in priority order |

### Executor (`executor/`)

| Type | File | Purpose |
|---|---|---|
| `EtheramExecutor` | `etheram_executor.rs` | Dispatches output actions (`SendMessage`, `BroadcastMessage`, `ScheduleTimer`, `RespondToClient`) |

### Observer (`observer.rs`)

| Type | Purpose |
|---|---|
| `Observer` trait | Per-action callbacks: `context_built`, `action_emitted`, `mutation_applied`, `output_executed` |
| `ActionKind` enum | Non-generic projection of `Action<M>` for object-safe observation |

### Collections (`collections/`)

| Type | Purpose |
|---|---|
| `ActionCollection` | `Vec`-backed `Collection` impl for protocol actions |

---

## Source Layout

```
src/
  lib.rs                    #![no_std], pub mod declarations
  etheram_node.rs           EtheramNode struct + step loop
  observer.rs               Observer trait + ActionKind enum
  brain/                    Protocol type alias (BoxedProtocol)
  collections/              ActionCollection
  common_types/
    account.rs              Account (balance, nonce)
    block.rs                Block struct + compute_hash
    transaction.rs          Transaction struct
    types.rs                Address, Hash, PeerId aliases
    state_root.rs           compute_state_root, compute_state_root_with_contract_storage
    *_adapter.rs            8 dimension adapter traits
  context/
    context_builder.rs      ContextBuilder trait
    context_dto.rs          ContextDto struct
  execution/
    execution_engine.rs     ExecutionEngine trait + BoxedExecutionEngine
    execution_result.rs     ExecutionResult
    transaction_result.rs   TransactionResult enum
    transaction_receipt.rs  TransactionReceipt struct
    receipts_root.rs        compute_receipts_root
    block_commitments.rs    compute_block_commitments
  executor/
    etheram_executor.rs     EtheramExecutor
    outgoing/               Per-dimension outgoing dispatch
  incoming/
    incoming_sources.rs     IncomingSources (priority polling)
    timer/                  TimerEvent enum
    transport/              TransportMessage, MessageSource
    external_interface/     ClientRequest, ClientResponse
  partitioner/
    partition.rs            Partitioner trait
  state/
    etheram_state.rs        EtheramState struct
    storage/                StorageQuery, StorageMutation
    cache/                  CacheQuery, CacheMutation
```

---

## Tests

21 tests in `etheram/tests/`:

| File | Count | Scope |
|---|---|---|
| `block_tests.rs` | 3 | `Block::compute_hash` determinism |
| `account_tests.rs` | 4 | `Account` construction and equality |
| `state_root_tests.rs` | 5 | `compute_state_root` determinism, order independence, contract storage |
| `receipts_root_tests.rs` | 5 | `compute_receipts_root` determinism |
| `transaction_receipt_tests.rs` | 4 | `TransactionReceipt` construction |

Integration tests that require concrete implementations (e.g., `IbftProtocol`, `InMemoryStorage`) live in [etheram-variants](../etheram-variants/README.md) and [etheram-validation](../etheram-validation/README.md), not here — enforcing the one-way dependency direction.
