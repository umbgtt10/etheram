# etheram-variants

> Concrete implementations, IBFT consensus protocol, and builder API

`etheram-variants` provides all concrete implementations of the traits defined in [etheram](../etheram/README.md): storage backends, transport layers, execution engines, the IBFT consensus protocol, and the builder API that wires them together. This is where the abstract architecture meets real behavior.

This crate is `#![cfg_attr(not(feature = "std"), no_std)]` — it compiles for both `std` (testing) and `no_std` (embedded) targets.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [etheram](../etheram/README.md)
**Depended on by:** [etheram-validation](../etheram-validation/README.md), [etheram-embassy](../etheram-embassy/README.md)

---

## Contents

### IBFT Consensus Protocol (`implementations/ibft/`)

Full Istanbul BFT implementation behind the `ConsensusProtocol` trait. See [IBFT-ROADMAP.md](../etheram/IBFT-ROADMAP.md) for feature tables.

| File | Purpose |
|---|---|
| `ibft_protocol.rs` | `IbftProtocol<S: SignatureScheme>` — main protocol struct with `handle_message` dispatch |
| `ibft_protocol/ibft_protocol_dispatch.rs` | Message routing (PrePrepare, Prepare, Commit, ViewChange, NewView, Timer, Client) |
| `ibft_protocol/ibft_protocol_validation.rs` | Block validation: `valid_block`, `valid_transactions`, commitment re-execution |
| `ibft_protocol/ibft_protocol_message_security.rs` | Message authentication and deduplication |
| `ibft_message.rs` | `IbftMessage` enum (PrePrepare, Prepare, Commit, ViewChange, NewView) |
| `validator_set.rs` | `ValidatorSet` — quorum computation (`⌊2n/3⌋ + 1`), proposer rotation |
| `vote_tracker.rs` | `VoteTracker` — per-phase vote counting with quorum detection |
| `prepared_certificate.rs` | `PreparedCertificate` — cryptographic quorum proof (block hash + signed prepares) |
| `signature_scheme.rs` | `SignatureScheme` trait + `SignatureBytes` newtype |
| `mock_signature_scheme.rs` | `MockSignatureScheme` — zeroed sigs, always-true verify (testing) |
| `ed25519_signature_scheme.rs` | `Ed25519SignatureScheme` — real `ed25519-dalek` signing/verification |
| `consensus_wal.rs` | `ConsensusWal` — write-ahead log for crash recovery |
| `wal_writer.rs` | `WalWriter` trait — abstraction for WAL persistence |
| `validator_set_update.rs` | `ValidatorSetUpdate` — height-gated validator transitions |

### Execution Engines (`implementations/`)

| Engine | File | Purpose |
|---|---|---|
| `TinyEvmEngine` | `tiny_evm_engine.rs` | Subset EVM: PUSH1, ADD, MUL, SSTORE, SLOAD, RETURN + per-opcode gas |
| `ValueTransferEngine` | `value_transfer_engine.rs` | Balance transfers only, intrinsic gas |
| `NoOpExecutionEngine` | `no_op_execution_engine.rs` | Returns empty results (testing/empty blocks) |

Supporting files:
- `tiny_evm_gas.rs` — gas cost constants (post-Istanbul schedule)
- `tiny_evm_opcode.rs` — opcode constants

### Infrastructure Implementations (`implementations/`)

| Type | File | Purpose |
|---|---|---|
| `InMemoryStorage` | `in_memory_storage.rs` | `BTreeMap`-backed storage with auto state root recomputation |
| `InMemoryCache` | `in_memory_cache.rs` | `Vec`-backed pending transaction pool |
| `InMemoryTimer` | `in_memory_timer.rs` | Manual timer with `schedule` and `push_event` |
| `InMemoryTransport` | `in_memory_transport.rs` | Shared-state transport via `Arc<Mutex<...>>` |
| `InMemoryExternalInterface` | `in_memory_external_interface.rs` | Push/drain request/response queues |
| `EagerContextBuilder` | `eager_context_builder.rs` | Reads full account map + state root from storage |
| `TypeBasedPartitioner` | `type_based_partitioner.rs` | Classifies actions by variant (mutation vs output vs execution) |
| `SharedState` | `shared_state.rs` | `Arc<Mutex<...>>` transport state for cluster wiring |

#### No-Op Implementations (testing stubs)

| Type | Purpose |
|---|---|
| `NoOpProtocol` | Returns empty actions |
| `NoOpTransport` | Discards all messages |
| `NoOpExternalInterface` | Never polls, discards responses |
| `NoOpTimer` | Never fires, discards schedules |
| `NoOpObserver` | Ignores all callbacks |

### Variant Enums (`variants.rs`)

Enums that enumerate all available implementations for each swappable slot:

| Enum | Variants |
|---|---|
| `StorageVariant` | `InMemory` |
| `CacheVariant` | `InMemory` |
| `ProtocolVariant` | `NoOp`, `Ibft` |
| `PartitionerVariant` | `TypeBased` |
| `ContextBuilderVariant` | `Eager` |
| `ExecutionEngineVariant` | `NoOp`, `ValueTransfer`, `TinyEvm` |
| ... | (one per swappable dimension) |

### Builder API (`builders/`)

Per-component builders that construct trait objects from variant enums:

| Builder | File | Builds |
|---|---|---|
| `EtheramNodeBuilder` | `etheram_node_builder.rs` | Complete `EtheramNode` from component builders |
| `StorageBuilder` | `storage_builder.rs` | `Box<dyn StorageAdapter>` |
| `CacheBuilder` | `cache_builder.rs` | `Box<dyn CacheAdapter>` |
| `ProtocolBuilder` | `protocol_builder.rs` | `BoxedProtocol` |
| `PartitionerBuilder` | `partitioner_builder.rs` | `Box<dyn Partitioner>` |
| `ContextBuilderBuilder` | `context_builder_builder.rs` | `Box<dyn ContextBuilder>` |
| `ExecutionEngineBuilder` | `execution_engine_builder.rs` | `BoxedExecutionEngine` |
| `ObserverBuilder` | `observer_builder.rs` | `Box<dyn Observer>` |
| `TimerInputBuilder` | `timer_input_builder.rs` | Timer input adapter |
| `TimerOutputBuilder` | `timer_output_builder.rs` | Timer output adapter |
| `TransportIncomingBuilder` | `transport_incoming_builder.rs` | Transport incoming adapter |
| `TransportOutgoingBuilder` | `transport_outgoing_builder.rs` | Transport outgoing adapter |
| `ExternalInterfaceIncomingBuilder` | `external_interface_incoming_builder.rs` | EI incoming adapter |
| `ExternalInterfaceOutgoingBuilder` | `external_interface_outgoing_builder.rs` | EI outgoing adapter |

---

## Source Layout

```
src/
  lib.rs                   #![cfg_attr(not(feature = "std"), no_std)], pub mod declarations
  variants.rs              Variant enums for each swappable slot
  builders/
    etheram_node_builder.rs
    storage_builder.rs
    cache_builder.rs
    protocol_builder.rs
    partitioner_builder.rs
    context_builder_builder.rs
    execution_engine_builder.rs
    observer_builder.rs
    timer_input_builder.rs / timer_output_builder.rs
    transport_incoming_builder.rs / transport_outgoing_builder.rs
    external_interface_incoming_builder.rs / external_interface_outgoing_builder.rs
    error.rs
  implementations/
    ibft/
      ibft_protocol.rs + ibft_protocol/     IBFT consensus
      ibft_message.rs                       Message types
      validator_set.rs                      Quorum + proposer logic
      vote_tracker.rs                       Per-phase vote counting
      prepared_certificate.rs               Cryptographic quorum proof
      signature_scheme.rs                   SignatureScheme trait
      mock_signature_scheme.rs              Mock (testing)
      ed25519_signature_scheme.rs           Real crypto
      consensus_wal.rs                      WAL serialization
      wal_writer.rs                         WalWriter trait
      validator_set_update.rs               Height-gated transitions
    tiny_evm_engine.rs                      Subset EVM execution
    tiny_evm_gas.rs                         Gas constants
    tiny_evm_opcode.rs                      Opcode constants
    value_transfer_engine.rs                Balance transfer engine
    no_op_execution_engine.rs               No-op engine
    in_memory_storage.rs                    In-memory storage
    in_memory_cache.rs                      In-memory cache
    in_memory_timer.rs                      In-memory timer
    in_memory_transport.rs                  In-memory transport
    in_memory_external_interface.rs         In-memory external interface
    eager_context_builder.rs                Eager context builder
    type_based_partitioner.rs               Type-based partitioner
    shared_state.rs                         Shared transport state
    no_op_*.rs                              No-op stubs
```

---

## Tests

391 tests across 52 test files in `tests/`:

### Builder Tests (14 files)
One test file per builder, verifying construction from variant enums.

### Implementation Tests (17 files)
Unit tests for each concrete implementation:
- `in_memory_storage_tests.rs` (7 tests)
- `in_memory_timer_tests.rs` (5 tests)
- `in_memory_transport_tests.rs` (5 tests)
- `in_memory_external_interface_tests.rs` (5 tests)
- `type_based_partitioner_tests.rs` (3 tests)
- `eager_context_builder_tests.rs`, `etheram_executor_tests.rs`, `etheram_state_tests.rs`
- `tiny_evm_execution_engine_tests.rs`, `tiny_evm_gas_tests.rs`, `tiny_evm_opcode_tests.rs`
- `value_transfer_engine_tests.rs` (8 tests)
- `no_op_execution_engine_tests.rs`, `no_op_observer_tests.rs`
- `transaction_receipt_storage_tests.rs`

### IBFT Protocol Tests (21 files)
Per-behavior test files covering every protocol path:

| File | Scope |
|---|---|
| `ibft_protocol_propose_tests.rs` | Timer-driven block proposal |
| `ibft_protocol_pre_prepare_tests.rs` | PrePrepare validation and acceptance |
| `ibft_protocol_prepare_tests.rs` | Prepare accumulation and quorum |
| `ibft_protocol_commit_tests.rs` | Commit phase and block finalization |
| `ibft_protocol_view_change_tests.rs` | View change and NewView handling |
| `ibft_protocol_client_tests.rs` | Client request/response flow |
| `ibft_protocol_persistence_tests.rs` | WAL save/restore |
| `ibft_protocol_replay_tests.rs` | Message replay prevention |
| `ibft_protocol_dedup_tests.rs` | Duplicate message filtering |
| `ibft_protocol_injection_tests.rs` | Byzantine signature injection |
| `ibft_protocol_malicious_block_tests.rs` | Conflicting block rejection |
| `ibft_protocol_signature_tests.rs` | Ed25519 sign/verify integration |
| `ibft_protocol_future_buffer_tests.rs` | Future-round message buffering |
| `ibft_protocol_reexecution_tests.rs` | Block re-execution validation |
| `ibft_protocol_validator_set_update_tests.rs` | Height-gated validator transitions |
| `ibft_protocol_wal_writer_tests.rs` | WAL writer abstraction |
| `validator_set_tests.rs` | Quorum size, proposer rotation |
| `vote_tracker_tests.rs` | Vote counting and quorum detection |
| `mock_signature_scheme_tests.rs` | Mock scheme behavior |
| `ed25519_signature_scheme_tests.rs` | Ed25519 sign/verify |
| `consensus_wal_tests.rs` | WAL serialization round-trip |

All tests are organized under `tests/all_tests.rs` as a single integration test entry point, mirroring the `src/` directory structure.
