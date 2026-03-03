# EtheRAM Chain — Feature Status

**Scope:** Ethereum-like chain semantics in EtheRAM
**Implementation:** `etheram/src/` (types, traits) + `etheram-variants/src/implementations/` (concrete engines, storage)
**Related:** [IBFT Roadmap](IBFT-ROADMAP.md) — Consensus protocol features

---

## Supported Features

### Block and State Model

| Feature | Description | Tests |
|---|---|---|
| Block structure | Height, proposer, transactions, `state_root`, `post_state_root`, `receipts_root` | ✅ |
| Block hash | Deterministic `compute_hash()` over all block fields | ✅ |
| Account model | Balance + nonce per `Address` (`[u8; 20]`) | ✅ |
| State root | Deterministic XOR-mix hash over sorted `BTreeMap<Address, Account>` | ✅ |
| State root with contract storage | Incorporates contract storage slots into state root | ✅ |
| Genesis accounts | Pre-seeded accounts with initial balances at height 0 | ✅ |
| Auto state root recomputation | `InMemoryStorage` and `SemihostingStorage` recompute on every `UpdateAccount` | ✅ |

### Transaction Lifecycle

| Feature | Description | Tests |
|---|---|---|
| Transaction structure | From, to, value, gas_limit, nonce, data (bytecode) | ✅ |
| Client submission | `ClientRequest::SubmitTransaction` → pending pool | ✅ |
| Nonce validation | Reject transactions with wrong nonce (`InvalidNonce` response) | ✅ |
| Balance validation | Reject insufficient balance (`InsufficientBalance` response) | ✅ |
| Gas limit validation | Reject gas_limit > `MAX_GAS_LIMIT` (`GasLimitExceeded` response) | ✅ |
| Block inclusion | Valid pending transactions included in proposed blocks | ✅ |
| Transaction application on commit | `UpdateAccount` mutations (sender balance−, nonce+1; receiver balance+) emitted per committed tx | ✅ |
| Pending pool cleanup | `RemovePending` cache mutation after commit | ✅ |

### Execution Engines

| Engine | Description | Gas Model | Tests |
|---|---|---|---|
| `TinyEvmEngine` | Subset EVM with opcode execution | Per-opcode gas (post-Istanbul schedule) | ✅ |
| `ValueTransferEngine` | Balance transfers only | Intrinsic gas (21000) | ✅ |
| `NoOpExecutionEngine` | Returns empty results | None | ✅ |

All engines implement the `ExecutionEngine` trait: immutable input, declarative `ExecutionResult`, no I/O. Engines are swappable at node construction time.

### TinyEVM Opcodes

| Opcode | Gas Cost | Description |
|---|---|---|
| `PUSH1` | 3 | Push 1-byte value onto stack |
| `ADD` | 3 | Addition |
| `MUL` | 5 | Multiplication |
| `SSTORE` | 20 000 | Store value at storage slot |
| `SLOAD` | 800 | Load value from storage slot |
| `RETURN` | 0 | Halt and return |
| Unknown | — | Returns `OutOfGas` |

Gas constants are aligned with the post-Istanbul EVM schedule.

### Gas Metering

| Feature | Description | Tests |
|---|---|---|
| Intrinsic gas | 21 000 per transaction (both engines) | ✅ |
| Per-opcode gas | Each TinyEVM opcode deducts cost before execution | ✅ |
| Out-of-gas | Transaction reverts with `OutOfGas` status; balance preserved | ✅ |
| `MAX_GAS_LIMIT` | Protocol-level cap (1 000 000); exceeding → `GasLimitExceeded` | ✅ |
| Transaction receipts | `TransactionReceipt` per transaction: `status`, `gas_used`, `cumulative_gas_used` | ✅ |
| Receipts root | Deterministic hash over receipt list | ✅ |
| `StoreReceipts` mutation | Computes real success/out_of_gas counts from receipt statuses | ✅ |
| Mixed success/failure | Block with mix of passing and OOG transactions commits correctly | ✅ |

### Block Re-execution (Commitment Validation)

| Feature | Description | Tests |
|---|---|---|
| Proposer commitments | Proposer computes `post_state_root` + `receipts_root` via execution engine | ✅ |
| Validator re-execution | Validators re-execute the block and compare commitments before voting | ✅ |
| Centralized computation | `compute_block_commitments()` — single source of truth for both roles | ✅ |
| Multi-height consistency | Commitments remain correct across consecutive heights with evolving state | ✅ |

### Contract Storage

| Feature | Description | Tests |
|---|---|---|
| `SSTORE` / `SLOAD` | Per-contract key-value storage slots | ✅ |
| Storage mutations | `UpdateContractStorage` mutations emitted by execution engine | ✅ |
| State root inclusion | Contract storage incorporated into state root computation | ✅ |

### Storage Backends

| Backend | Crate | `no_std` | Description |
|---|---|---|---|
| `InMemoryStorage` | `etheram-variants` | ✅ | `BTreeMap`-backed, auto state root recomputation |
| `SemihostingStorage` | `etheram-embassy` | ✅ | ARM semihosting file I/O for QEMU persistence |

### Client Interface

| Feature | Description | Tests |
|---|---|---|
| `SubmitTransaction` | Client submits a transaction; receives `TransactionAccepted` or rejection | ✅ |
| `QueryBalance` | Client queries account balance | ✅ |
| `QueryHeight` | Client queries committed chain height | ✅ |
| `QueryContractStorage` | Client queries a contract storage slot | ✅ |

### Observability

| Feature | Description | Tests |
|---|---|---|
| `Observer` trait | Per-action callbacks: `context_built`, `action_emitted`, `mutation_applied`, `output_executed` | ✅ |
| `ActionKind` enum | Non-generic projection of `Action<M>` for `Box<dyn Observer>` object safety | ✅ |
| `SemihostingObserver` | Embassy logging via ARM semihosting with per-item detail | ✅ (QEMU) |
| `NoOpObserver` | Silent (testing) | ✅ |

### Swappable Components

| Slot | Trait | Implementations |
|---|---|---|
| Storage | `StorageAdapter` | `InMemoryStorage`, `SemihostingStorage` |
| Cache | `CacheAdapter` | `InMemoryCache` |
| Transport | `TransportOutgoingAdapter` | `InMemoryTransport`, `NoOpTransport`, `OutboxTransport`, UDP |
| Timer | `TimerInputAdapter` + `TimerOutputAdapter` | `InMemoryTimer`, `NoOpTimer`, Embassy channel |
| External Interface | `ExternalInterfaceIncomingAdapter` + `...Outgoing` | `InMemoryExternalInterface`, `NoOpExternalInterface`, Embassy channel, UDP |
| Context Builder | `ContextBuilder` | `EagerContextBuilder` |
| Partitioner | `Partitioner` | `TypeBasedPartitioner` |
| Execution Engine | `ExecutionEngine` | `TinyEvmEngine`, `ValueTransferEngine`, `NoOpExecutionEngine` |
| Signature Scheme | `SignatureScheme` | `MockSignatureScheme`, `Ed25519SignatureScheme` |
| Observer | `Observer` | `NoOpObserver`, `SemihostingObserver` |

---

## Planned Features

### Chain Semantics

| Feature | Description | Priority | Complexity |
|---|---|---|---|
| Merkle Patricia Trie | Cryptographically verifiable state root (replaces XOR-mix) | High | High |
| Full EVM opcode set | Complete EVM instruction set (Ethereum Execution Spec Tests) | Medium | Very High |
| Contract deployment | `CREATE` / `CREATE2` opcode support | Medium | High |
| EIP-2930 access lists | Pre-declared storage slot reads for warm/cold gas accounting | Low | Medium |
| Block gas limit | Cumulative gas limit per block | Medium | Low |
| Transaction fees | Gas price × gas used deducted from sender | Medium | Low |
| State snapshots | Point-in-time state capture for rollback/audit | Low | Medium |

### Infrastructure

| Feature | Description | Priority | Complexity |
|---|---|---|---|
| Persistent storage (RocksDB) | Disk-backed storage for production use | Medium | Medium |
| TCP transport | Real TCP peer-to-peer networking | Medium | Medium |
| System timer | Wall-clock timer (replacing `InMemoryTimer`) | Low | Low |
| JSON-RPC interface | Ethereum-compatible client API | Low | High |
| Peer discovery | Dynamic peer joining/leaving | Low | High |

### Testing and Verification

| Feature | Description | Priority | Complexity |
|---|---|---|---|
| Property-based tests (`proptest`) | Randomized invariant testing for `handle_message` | High | Medium |
| Forks and reorgs | State rollback under competing blocks | Medium | High |
| Transactional execution | Batch action execution with commit/rollback | Low | Medium |
| Parallel execution | Independent actions executed concurrently | Low | Medium |
| Physical hardware deployment | STM32 / RP2040 with real UDP and flash storage | Medium | High |
| Formal specification (TLA+) | Machine-checked proof of protocol invariants | Medium | Very High |

