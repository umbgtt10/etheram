# Raft vs Etheram Architectural Review

## 1. Executive Summary

The `raft-*` ecosystem successfully mirrors the 3-6 architectural decomposition of the reference `etheram-*` implementation. The core node logic (`raft-node`) is structurally sound, `no_std` compatible, and correctly implements the "Brain-Input-Output-State" separation.

However, `raft-variants` significantly diverges from `etheram-variants` in the **Construction Pattern**. While Etheram uses a "Variant Enum + Component Builder" pattern (allowing configuration-driven instantiation), Raft currently relies on a "Dependency Injection" builder (requiring manual instantiation of concrete types).

## 2. Structural & Naming Consistency

| Aspect | Etheram (Reference) | Raft (Implementation) | Status |
|---|---|---|---|
| **Crate Structure** | `etheram`, `etheram-variants`, `etheram-validation` | `raft-node`, `raft-variants`, `raft-validation` | ✅ Matched |
| **Node Generics** | `EtheramNode<M>` (Message) | `RaftNode<P>` (Payload) | ✅ Matched |
| **Step Loop** | `incoming` -> `context` -> `brain` -> `partitioner` -> `executor` | Identical flow | ✅ Matched |
| **Partitioning** | 3-way: `mutations`, `outputs`, `executions` | 2-way: `mutations`, `outputs` (SM application is inline) | ⚠️ Minor Divergence |
| **State Separation** | `EtheramState` (Storage+Cache) vs `ExecutionEngine` | `RaftState` (Storage+Cache) vs `StateMachine` | ✅ Matched |

## 3. Discovered Inconsistencies & Deficiencies

### 3.1. The Component Builder Gap (Major)
`etheram-variants` provides a robust set of component builders (`StorageBuilder`, `TimerBuilder`, etc.) driven by enums (`StorageVariant`, `TimerVariant`). This allows a node to be configured declaratively:
```rust
// Etheram (Conceptual)
let storage = StorageBuilder::new().with_variant(StorageVariant::InMemory).build();
```

`raft-variants` **lacks** these component builders and variant enums. It only provides `RaftNodeBuilder`, which accepts pre-built `Box<dyn ...>` traits.
```rust
// Raft (Current)
let storage = Box::new(InMemoryRaftStorage::new()); // Manual instantiation required
builder.with_storage(storage);
```
**Impact**: Raft cannot be easily configured via a config file or CLI flags without writing manual wiring code for every combination.

### 3.2. Step Loop Partitioning Strategy
`EtheramNode` explicitly partitions actions into three buckets: `mutations`, `outputs`, and `executions`.
`RaftNode` partitions into `mutations` and `outputs`, but then manually scans `outputs` for `ApplyToStateMachine` actions and executes them synchronously before the main executor.
**Recommendation**: This is acceptable given Raft's simpler execution model, but strictly speaking, `ApplyToStateMachine` behaves like an `Execution` separation.

### 3.3. Etheram Builder Generic Flaw (Side Finding)
The `EtheramNodeBuilder` in `etheram-variants` appears to be hardcoded to `BoxedProtocol<()>` (Unit message type). This makes it unusable for `IbftProtocol` (which uses `IbftMessage`). This observation explains why `IbftTestNode` manually constructs the node instead of using the builder.
**Recommendation**: The `EtheramNodeBuilder` needs to be made generic `EtheramNodeBuilder<M>` to be useful for protocols other than unit-tests.

## 4. Redundancies
*   No significant code redundancies found. The duplication of standard traits (`StorageAdapter`, etc.) across `etheram` and `raft-node` is intentional to keep the ecosystems independent (`no_std` + clean dependency tree).

## 5. Proposed Improvements

### Immediate Actions (Raft)
1.  **Implement Variants Enums**: Create `raft-variants/src/variants.rs` with `RaftStorageVariant`, `RaftTransportVariant`, etc.
2.  **Implement Component Builders**: Port `StorageBuilder`, `TransportBuilder`, etc., from `etheram-variants` to `raft-variants`.
3.  **Update Node Builder**: simple `with_storage` should accept `Box<dyn Storage...>`, but `with_storage_variant` should accept the enum.

### Future Actions (Etheram)
1.  **Fix Generic Builder**: Refactor `EtheramNodeBuilder` to `EtheramNodeBuilder<M>` to support IBFT construction properly.

## 6. Code Quality
*   **Safety**: Both use `Box<dyn Trait>` heavily, which is correct for swappability.
*   **Standards**: Both adhere to strict `no_std` separation (verified via imports).
*   **Testing**: Raft test suite is comprehensive (protocol + cluster), mirroring the Etheram validation strategy.

## 7. Conclusion
Raft is a high-quality implementation of the 3-6 model. The structural foundations are perfect. The only "debt" is the lack of variant-driven builders in `raft-variants`, which positions it slightly behind `etheram-variants` in terms of configuration ergonomics.

**Grade: A-** (Excellent Core, Missing Configuration Layer)
