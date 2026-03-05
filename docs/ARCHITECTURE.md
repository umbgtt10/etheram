# EtheRAM Architecture — The 3-6 Model

> Maximal abstraction for distributed system nodes

---

## Core Philosophy

EtheRAM is built on a single principle:

**Define WHAT is required, not HOW it is implemented.**

The architecture separates:
- **Contract** — What capabilities a blockchain node must provide
- **Strategy** — How those capabilities are realized

This separation enables radical experimentation while maintaining correctness guarantees. Etheram and Raft validate this: the same node architecture and `step()` primitive run identically in std test harnesses, multi-node cluster orchestrators, and no_std ARM Cortex-M4 embedded tasks — with zero changes to core node logic.

---

## The Node Abstraction

A blockchain node is decomposed into **six orthogonal dimensions**:

1. **Protocol** — The consensus algorithm (stateless, pure functions)
2. **Storage** — Persistent state (survives crashes)
3. **Cache** — Volatile working state (algorithm-specific)
4. **Transport** — Peer-to-peer communication
5. **ExternalInterface** — Client communication (RPC/API layer)
6. **Timer** — Time-based event scheduling

### The Etheram Realization

`EtheramNode<M>` is the concrete realization of this decomposition:

```rust
pub struct EtheramNode<M: Clone + 'static> {
    peer_id: PeerId,
    incoming: IncomingSources<M>,        // polls Timer, ExternalInterface, Transport
    state: EtheramState,                 // wraps Storage + Cache
    executor: EtheramExecutor<M>,        // dispatches output actions
    context_builder: Box<dyn ContextBuilder<M>>,
    brain: BoxedProtocol<M>,             // consensus protocol (pure)
    partitioner: Box<dyn Partitioner<M>>,
    execution_engine: BoxedExecutionEngine,
    observer: Box<dyn Observer>,
}
```

All decision components (`ContextBuilder`, `Protocol`, `Partitioner`, `ExecutionEngine`) and all infrastructure dimensions (`Storage`, `Cache`, `Transport`, `ExternalInterface`, `Timer`) are swappable via trait objects. The `Observer` provides per-step visibility into every dimension interaction.

### What the Architecture Does NOT Prescribe

- Whether types are generic, dynamic (`Box<dyn Trait>`), or concrete
- Whether dimensions are swappable at runtime or compile-time
- Performance characteristics
- Memory layout
- Ownership model
- Execution environment (std, no_std, async, sync)

---

## Execution Model: `step()` as Primitive

The core execution primitive is a single synchronous method:

```rust
pub fn step(&mut self) -> bool;
```

**Properties:**
- **Synchronous** — Returns immediately, never blocks
- **Deterministic** — Same events produce same state changes
- **Minimal** — Processes exactly one event (or returns false if idle)
- **Observable** — Observer notified at each phase (context built, action emitted, mutation applied, output executed, step completed)
- **Testable** — Can be called step-by-step with full control

**The step loop:**
1. Poll event sources (`IncomingSources::poll()`) in deterministic order: Timer → ExternalInterface → Transport
2. Build immutable context via `ContextBuilder` (snapshot of current state)
3. Pass event to protocol (`brain.handle_message()`) — pure computation, returns declarative actions
4. Partition actions via `Partitioner` into mutations, outputs, and executions
5. Apply mutations to state (`EtheramState::apply_mutations()`)
6. Dispatch outputs to executor (`EtheramExecutor::execute_outputs()`)
7. Run execution engine for block executions (transaction results, receipts, contract storage updates)

**All execution patterns build on `step()`:**
- Blocking loops: `while node.step() { }`
- Run-until-idle: `while node.step() {}` — the standard pattern
- Deterministic testing: `cluster.fire_timer(0, ProposeBlock); cluster.step_all();`
- Embedded async: Embassy `select4` reactor awaiting events, then `while node.step() {}`

Methods like `run()`, `shutdown()`, async wrappers are convenience implementations, not core abstractions.

### Operating Models: Implementation Detail

The architecture does NOT prescribe:

**Concurrency Model:**
- Sequential (single-threaded, loop-based)
- Task-based (async/await, tokio/Embassy tasks)
- Thread-based (OS threads, thread pool)
- Process-based (multi-process, IPC)
- Actor-based (message passing between actors)

**Scheduling Strategy:**
- Queue-based (FIFO event queue)
- Priority-based (weighted event sources)
- Round-robin (fair scheduling across dimensions)
- Deadline-driven (real-time constraints)
- Work-stealing (multi-threaded executors)

**Event Coordination:**
- Polling-based (non-blocking checks, `IncomingSources::poll()` design)
- Interrupt-driven (hardware/software interrupts)
- Callback-based (event listeners)
- Channel-based (MPSC/MPMC queues, Embassy channels)

**Validated implementations:**

```
Sequential Testing (etheram-validation):
  cluster.fire_timer(node_idx, TimerEvent::ProposeBlock);
  cluster.step_all();  // while node.step() {} on each node

Cluster Orchestration (etheram-validation):
  cluster.step(0);     // step single node
  cluster.step_all();  // step all nodes to idle

Embassy Embedded (etheram-embassy):
  #[embassy_executor::task(pool_size = 5)]
  async fn node_task(mut node: EtheramNode<IbftMessage>, ...) {
      loop {
          match select4(
              cancel.wait(),
              transport_receiver.receive(),
              timer_receiver.receive(),
              ei_notify.receive(),
          ).await {
              Either4::First(()) => break,
              Either4::Second((from, msg)) => {
                  transport_state.push_message(peer_id, from, msg);
                  while node.step() {}
              }
              // ... timer, external interface similarly
          }
      }
  }
```

**The abstraction is agnostic to all of these choices.** The same `EtheramNode::step()` method is used in all three environments.

---

## Flexibility Spectrum

### Three Implementation Strategies

1. **Static (Compile-time)**
   - Generic type parameters
   - Zero runtime cost
   - Monomorphized at compile time
   - Cannot swap at runtime

2. **Dynamic (Runtime)**
   - `Box<dyn Trait>` trait objects
   - Runtime swappable
   - Vtable dispatch overhead
   - Heap allocations

3. **Concrete**
   - Specific named types
   - Maximum performance
   - No swapping
   - No abstraction overhead

### Per-Dimension Independence

**Each dimension independently chooses its strategy.**

Etheram uses dynamic dispatch (`Box<dyn Trait>`) for maximum swappability:
- Protocol: `BoxedProtocol<M>` — swap `IbftProtocol<MockSignatureScheme>` ↔ `IbftProtocol<Ed25519SignatureScheme>` ↔ `NoOpProtocol`
- Storage: `Box<dyn StorageAdapter>` — swap `InMemoryStorage` ↔ `SemihostingStorage`
- Cache: `Box<dyn CacheAdapter>` — swap `InMemoryCache`
- Transport: `Box<dyn TransportIncomingAdapter>` / `Box<dyn TransportOutgoingAdapter>` — swap `InMemoryTransport` ↔ `OutboxTransport` ↔ `UdpIbftTransport` ↔ `NoOpTransport`
- ExternalInterface: `Box<dyn ExternalInterfaceIncomingAdapter>` — swap `InMemoryExternalInterface` ↔ channel-based ↔ UDP-based
- Timer: `Box<dyn TimerInputAdapter>` — swap `InMemoryTimer` ↔ Embassy timer receivers
- Execution Engine: `BoxedExecutionEngine` — swap `ValueTransferEngine` ↔ `TinyEvmEngine`

This creates a **flexibility/performance tradeoff at the dimension level**, not system level. Embassy QEMU validation confirms that vtable dispatch overhead is acceptable even on embedded ARM Cortex-M4.

### Mixed Strategies in Production

Real implementations commonly mix strategies:
```
EtheramNode (current):
  protocol: BoxedProtocol<IbftMessage>           (dynamic - swap signature scheme)
  storage: Box<dyn StorageAdapter>               (dynamic - InMemory vs Semihosting)
  cache: Box<dyn CacheAdapter>                   (dynamic)
  transport: Box<dyn TransportIncomingAdapter>      (dynamic - InMemory vs UDP vs channel)
  external_interface: Box<dyn ExternalInterface>  (dynamic - InMemory vs channel vs UDP)
  timer: Box<dyn TimerInputAdapter>              (dynamic)
  execution_engine: BoxedExecutionEngine         (dynamic - ValueTransfer vs TinyEvm)
```

---

## Design Decisions

### 1. No Separate Executor Dimension

Early designs considered a seventh dimension: "Executor."

**Analysis revealed:** With maximal abstraction, the executor collapses into the Node itself. The `step()` method IS the execution primitive. Event loops, async wrappers, and scheduling are implementation details, not core abstractions.

**Six dimensions, not seven.** The `EtheramExecutor` is an internal component that dispatches output actions to outgoing dimensions — it is not a separate dimension.

### 2. Implicit Dimension Access

The `step()` method does not take dimensions as parameters:
```rust
fn step(&mut self) -> bool;  // ✅ Correct
```

Not:
```rust
fn step(&mut self, protocol: &P, storage: &mut S, ...) -> bool;  // ❌ Wrong
```

**Rationale:** This would prescribe internal organization. The architecture defines capabilities, not structure. Etheram stores dimensions as struct fields with `Box<dyn Trait>` — but other realizations could proxy to remote services, compute on-demand, or share dimensions across nodes.

### 3. Unified Message Handling

Protocol handles ALL inputs (peer, client, timer) through a single entry point — `handle_message(source, message, context) → actions`. There are no separate handlers for consensus vs client vs timer. All three are consensus decisions routed through one function.

**Why it's better than the traditional approach** (separate consensus module + mempool module + block production module + shared state with locks):
- No artificial separation between "consensus" and "other concerns"
- Protocol can correlate events across dimensions
- No race conditions from concurrent module access
- Simpler testing (just call one function)
- Semantically honest (all three ARE consensus decisions)

### 4. Context Ownership

Context owns all data (no references/lifetimes). Built fresh for each message by cloning from dimensions. Protocol cannot accidentally observe mutations mid-execution.

```rust
pub struct Context {
    pub peer_id: PeerId,
    pub current_height: Height,
    pub state_root: Hash,
    pub accounts: BTreeMap<Address, Account>,
    pub contract_storage: BTreeMap<(Address, Hash), Hash>,
    pub pending_txs: Vec<Transaction>,
}
```

**Rationale:** `ConsensusProtocol` trait cannot have lifetime parameters on associated types. Immutable snapshots enable pure functional reasoning.

### 5. Actions as Pure Data

Protocol returns `ActionCollection`, execution is separate. Actions can be logged, queued, batched, or optimized. Etheram realizes this through the `Partitioner` + `EtheramExecutor` + `ExecutionEngine` chain.

### 6. Protocol as Pure Computation

Protocol is stateless (`&self`), no internal mutation. Testing becomes `protocol(context, message) == expected_actions` — just assert equality. No concurrency issues within protocol. Reusable across different operating models.

EtheRAM validates this with 564 deterministic tests covering all IBFT and Ethereum protocol paths — all tested via pure function calls with no infrastructure setup.

### 7. Action Partitioning

The `Partitioner` separates protocol actions into three categories:
- **Mutations** — State changes (`UpdateAccount`, `StoreBlock`, `IncrementHeight`, `UpdateCache`)
- **Outputs** — I/O effects (`BroadcastMessage`, `SendMessage`, `SendClientResponse`, `ScheduleTimeout`, `Log`)
- **Executions** — Block execution (`ExecuteBlock` — triggers execution engine for transaction processing, receipts, contract storage)

This enforces side-effect isolation: the protocol produces declarative actions; the node partitions and executes them in the appropriate order.

### 8. Protocol-Specific Types

Storage, Cache, Transport, ExternalInterface, and Timer all use protocol-specific types:
- `StorageQuery`, `StorageMutation` — Defined by protocol needs
- `CacheQuery`, `CacheUpdate` — Algorithm-specific state
- `PeerMessage` — Protocol messages (e.g. `IbftMessage` with `PrePrepare`, `Prepare`, `Commit`, `ViewChange`, `NewView`)
- `ClientRequest`, `ClientResponse` — API-specific (e.g. `SubmitTransaction`, `GetBalance`, `GetHeight`, `GetNonce`, `GetStateRoot`)
- `TimerEvent` — Protocol timeouts/scheduling (e.g. `ProposeBlock`, `TimeoutRound`)

**No generic event types.** Each protocol defines exactly what it needs.

---

## Abstraction Layers

### Layer 1: Infrastructure Traits (core)

Seven traits defining capabilities:
- `ConsensusProtocol` — Stateless protocol logic (`handle_message() → ActionCollection`)
- `StorageAdapter` — Persistent state (query/mutate with protocol-specific types)
- `CacheAdapter` — Volatile state (query/update/invalidate)
- `TransportIncomingAdapter` / `TransportOutgoingAdapter` — Peer communication (poll/send)
- `ExternalInterfaceIncomingAdapter` — Client communication (poll requests)
- `TimerInputAdapter` / `TimerOutputAdapter` — Time-based events (poll/schedule)
- `Collection` — Generic sequence abstraction (iteration, length, indexing)

**Responsibility:** Define interfaces, not implementations.

### Layer 2: Node Logic (etheram-node)

`EtheramNode<M>` composing the six dimensions:
- `IncomingSources<M>` — Polls Timer, ExternalInterface, Transport
- `EtheramState` — Wraps Storage + Cache
- `EtheramExecutor<M>` — Dispatches outputs to Transport, ExternalInterface, Timer
- `BoxedProtocol<M>` — Consensus protocol (pure decision logic)
- `Box<dyn ContextBuilder<M>>` — Builds immutable context snapshots
- `Box<dyn Partitioner<M>>` — Separates mutations from outputs from executions
- `BoxedExecutionEngine` — Transaction/contract execution
- `Box<dyn Observer>` — Per-step observability hooks

Plus common types: `Action<M>`, `MessageSource`, `Context`, `Transaction`, `Block`, `Account`, `TransactionReceipt`.

**Responsibility:** Define the node contract and step loop.

### Layer 3: Implementations (etheram-node)

Concrete types implementing the traits:
- **Protocol:** `IbftProtocol<S>` (IBFT BFT consensus, generic over `SignatureScheme`), `NoOpProtocol`
- **Storage:** `InMemoryStorage` (std/no_std, `BTreeMap`-backed with state root computation)
- **Cache:** `InMemoryCache` (pending transaction management)
- **Transport:** `InMemoryTransport` (shared state for cluster testing), `OutboxTransport` (sync-to-async bridge for Embassy), `UdpIbftTransport` (serialized UDP), `NoOpTransport`
- **ExternalInterface:** `InMemoryExternalInterface`, channel-based, UDP-based
- **Timer:** `InMemoryTimer` (deterministic test driving), Embassy async timer receivers
- **Execution Engine:** `ValueTransferEngine` (balance transfers), `TinyEvmEngine` (bytecode execution with gas metering)
- **Signature Scheme:** `MockSignatureScheme` (zero-cost testing), `Ed25519SignatureScheme` (production)
- **Context Builder:** `EagerContextBuilder` (reads all state eagerly)
- **Partitioner:** `TypeBasedPartitioner` (separates by action variant)
- **Observer:** `NoOpObserver`, `SemihostingObserver` (ARM semihosting logging)
- **Builders:** `EtheramNodeBuilder`, `StorageBuilder`, `ProtocolBuilder`, `PartitionerBuilder`, etc.

**Responsibility:** Realize the abstractions with specific strategies.

---

## Validation

The architecture is validated across three environments:

### Deterministic Testing (748 tests across both protocol families)
- `step()` primitive enables event-by-event control
- In-memory infrastructure implementations
- Deterministic time (controlled timer advancement via `push_event`)
- No async/await in core logic (synchronous poll-based)
- Byzantine fault injection, message interception, round interleaving

### Cluster Orchestration (205+ integration tests)
- `IbftCluster` with 4–7 validator nodes
- Shared in-memory transport enables controlled message delivery
- `step_all()` drives all nodes to idle; `step(n)` enables fine-grained interleaving
- Validates distributed correctness: consensus, view changes, validator set updates, transaction execution

### Embedded Deployment (QEMU, ARM Cortex-M4)
- Same `EtheramNode` and `IbftProtocol` running under Embassy async runtime
- 5-node IBFT consensus with Ed25519 signatures
- 12-act scenario: transfers, view changes, overdrafts, gas limits, validator set updates, WAL persistence, TinyEVM contract execution, OutOfGas reverts
- Two independently maintained configurations verified end-to-end
- Cross-environment proof: identical `etheram-node` crate logic compiles and executes correctly across std and no_std integration contexts

### Raft Validation (Protocol + Cluster + Embassy)
- `RaftNode<P>` mirrors the same six dimensions and `step()` execution model used by `EtheramNode`
- Protocol-level deterministic tests validate election, replication, snapshots, client handling, and role transitions
- Cluster-level deterministic tests validate fault tolerance, state-machine apply, snapshot installation, and client semantics
- Embassy deployment validates both required configurations end-to-end: all-in-memory and UDP+semihosting
- 5-act Raft scenario validated in QEMU: election, replication, read-after-write, re-election, continued replication

---

## Comparison to Traditional Architectures

### Traditional Blockchain Node
```
Monolithic implementation
├─ Hardcoded storage (e.g., LevelDB)
├─ Hardcoded networking (e.g., libp2p)
├─ Hardcoded consensus
└─ Tightly coupled components
```

**Problems:**
- Cannot test consensus without full networking stack
- Cannot swap components
- Validation requires running actual node
- One implementation per blockchain

### EtheRAM Node
```
Abstraction over six dimensions
├─ Any Storage (InMemoryStorage, SemihostingStorage, ...)
├─ Any Transport (InMemoryTransport, OutboxTransport, UdpIbftTransport, ...)
├─ Any Protocol (IbftProtocol<S>, NoOpProtocol, ...)
├─ Any Cache (InMemoryCache, ...)
├─ Any ExternalInterface (InMemoryExI, channel-based, UDP-based, ...)
└─ Any Timer (InMemoryTimer, Embassy timer, ...)
```

**Benefits:**
- Test protocols with in-memory infrastructure (748 deterministic tests)
- Swap components at construction via builder pattern
- Validation through deterministic simulation and cluster orchestration
- Same node logic deploys to std tests, cluster harnesses, and no_std embedded

---

## Why This Emerged

The 3-6 model was not designed upfront. It emerged from asking principled questions:

1. "Should protocol be testable?" → Pure function
2. "Should protocol work with any runtime?" → No async/threads in protocol
3. "Should we separate consensus from client handling?" → NO — it's all consensus decisions
4. "Can protocol have mutable state?" → NO — makes testing hard
5. "Should context have references?" → NO — trait lifetime constraints

Each decision forced the next, and the 3-6 model appeared as the natural consequence of principled choices.

---

## Resolved Questions

1. **Context building cost:** Cloning all data for each message — is this prohibitive at scale?
   - **Answer:** `EagerContextBuilder` clones eagerly. Acceptable at current scale (QEMU + 5 nodes). Future optimization path: `Arc`/`Rc` for immutable data, lazy loading, or copy-on-write. Not a blocking issue.

2. **Action granularity:** Should actions be fine-grained or coarse-grained?
   - **Answer:** Fine-grained. Actions include `UpdateAccount`, `StoreBlock`, `IncrementHeight`, `UpdateCache`, `BroadcastMessage`, `SendMessage`, `ExecuteBlock`, `ScheduleTimeout`, `Log`. The `Partitioner` classifies them into mutations, outputs, and executions.

3. **Error handling:** How should protocol signal errors?
   - **Answer:** Protocol returns actions — including `SendClientResponse` with error payloads like `InsufficientBalance`, `InvalidNonce`, `GasLimitExceeded`. No `Result` return; errors are just another kind of action.

4. **Async protocol?** Should `handle_message` be async?
   - **Answer:** No. Synchronous pure computation. Cryptographic operations (Ed25519 signing/verification) are fast enough to be synchronous. Async is the wrapper's concern, not the protocol's.

---

## Future Architecture Possibilities

### 1. Distributed Execution
```rust
struct RemoteExecutor {
    rpc_client: Client,
}

impl ActionExecutor for RemoteExecutor {
    fn execute(&mut self, actions: ActionCollection<Action>) {
        self.rpc_client.send(actions); // Execute on different machine
    }
}
```

### 2. Transactional State
```rust
struct TransactionalDimensionManager {
    storage: TransactionalStorage,
    cache: TransactionalCache,
}

impl DimensionManager for TransactionalDimensionManager {
    fn execute_actions(&mut self, actions: ActionCollection<Action>) {
        let tx = self.storage.begin_transaction();
        // Execute actions...
        tx.commit(); // Atomic
    }
}
```

### 3. Parallel Action Execution
```rust
impl DimensionManager for ParallelExecutor {
    fn execute_actions(&mut self, actions: ActionCollection<Action>) {
        let (storage_actions, io_actions): (Vec<_>, Vec<_>) =
            actions.partition(|a| matches!(a, Action::StoreBlock(_)));

        rayon::join(
            || self.execute_storage(storage_actions),
            || self.execute_io(io_actions)
        );
    }
}
```

### 4. Pipelined Processing
```rust
// While executing block N, verify block N+1, receive block N+2
struct PipelinedNode {
    verification_queue: Channel<Block>,
    execution_queue: Channel<ActionCollection>,
}
```

---

## Beyond Blockchain

The Node abstraction is actually a specialization of a more general **SystemEntity** pattern:

**SystemEntity properties:**
- Maintains state (storage + cache)
- Communicates with peers (transport)
- Communicates with clients (external interface)
- Reasons about time (timer)
- Executes a protocol
- Can fail and recover

**Applies to:**
- Blockchain validators (EtheRAM focus)
- BFT consensus nodes
- UAV swarms
- Autonomous vehicles
- Distributed controllers
- Robot coordination

The six-dimensional decomposition generalizes beyond blockchain to any distributed system participant.

### Generality Proven: Raft as Second Protocol

The 3-6 model is now validated across a second independent protocol family — **Raft** — implemented as its own crate family (`raft-node/`, `raft-validation/`, `raft-embassy/`). Raft is maximally different from IBFT across every axis: crash-only vs Byzantine fault model, `⌊n/2⌋+1` vs `⌊2n/3⌋+1` quorum, randomized vs deterministic leader election, 2-phase vs 3-phase commit, append-only log vs single pending block.

Despite these protocol-level differences, the same six-dimensional decomposition and the same `step()` primitive emerge unchanged, with `core/` as the shared abstraction layer and no cross-dependencies between protocol families. This is the architectural validation target of EtheRAM.

See [RAFT-ROADMAP.md](../etheram-node/RAFT-ROADMAP.md) for implementation details and milestone history.

---

## Key Takeaways

1. **Maximal Abstraction** — Core defines contracts (traits), not implementations
2. **Six Dimensions** — Protocol, Storage, Cache, Transport, ExternalInterface, Timer
3. **One Primitive** — `step()` is the single execution method
4. **Action Partitioning** — Mutations, outputs, and executions are separated and executed in order
5. **Pure Protocol** — Immutable context in, declarative actions out — no I/O
6. **Observability** — `Observer` trait provides per-step visibility into every phase
7. **Cross-Environment** — Same node logic validated across std tests, cluster harnesses, and no_std ARM embedded
8. **Implementation Freedom** — Radical variation in implementation strategy without changing core

This architecture achieves the project's core goal:

> Separate what must be correct from what may vary.
