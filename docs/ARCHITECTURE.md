# EtheRAM Architecture

> Maximal abstraction for distributed system nodes

---

## Core Philosophy

EtheRAM is built on a single principle:

**Define WHAT is required, not HOW it is implemented.**

The architecture separates:
- **Contract** — What capabilities a blockchain node must provide
- **Strategy** — How those capabilities are realized

This separation enables radical experimentation while maintaining correctness guarantees.

---

## The Node Abstraction

A blockchain node is decomposed into **six orthogonal dimensions**:

1. **Protocol** — The consensus algorithm (stateless, pure functions)
2. **Storage** — Persistent state (survives crashes)
3. **Cache** — Volatile working state (algorithm-specific)
4. **Transport** — Peer-to-peer communication
5. **ExternalInterface** — Client communication (RPC/API layer)
6. **Timer** — Time-based event scheduling

### The Contract

The `Node` trait defines:
- Six associated types (one per dimension)
- Trait bounds ensuring each dimension implements its infrastructure contract
- A single execution primitive: `step()`

### What the Trait Does NOT Prescribe

- How dimensions are stored (fields, references, handles, proxies)
- Whether types are generic, dynamic (`Box<dyn Trait>`), or concrete
- Whether dimensions are swappable at runtime
- Performance characteristics
- Memory layout
- Ownership model

---

## Execution Model: `step()` as Primitive

The core execution primitive is a single synchronous method:

```rust
fn step(&mut self) -> bool;
```

**Properties:**
- **Synchronous** — Returns immediately, never blocks
- **Deterministic** — Same events produce same state changes
- **Minimal** — Processes exactly one event (or returns false if idle)
- **Testable** — Can be called step-by-step with full control

**All execution patterns build on `step()`:**
- Blocking loops: `while node.step() { }`
- Async event-driven: `tokio::select!` on event sources, call `step()`
- Imperative testing: Manual calls with event injection
- Embedded/interrupt-driven: Timer-triggered `step()` calls

Methods like `run()`, `shutdown()`, async wrappers are convenience implementations, not core abstractions.

### Operating Models: Implementation Detail

The trait does NOT prescribe:

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
- Polling-based (non-blocking checks, current `poll()` design)
- Interrupt-driven (hardware/software interrupts)
- Callback-based (event listeners)
- Channel-based (MPSC/MPMC queues)

**Examples of valid implementations:**

```
Sequential Testing:
  loop { node.step(); }

Tokio Async:
  tokio::spawn(async move {
    loop {
      select! { /* await events */ }
      node.step();
    }
  })

Thread Pool:
  thread_pool.spawn(|| {
    while running { node.step(); }
  })

Embassy Embedded:
  #[embassy_executor::task]
  async fn node_task(mut node: Node) {
    loop {
      node.step();
      Timer::after_millis(10).await;
    }
  }

Priority Queue:
  loop {
    let priority = determine_priority();
    if priority == High { node.step(); }
    else { sleep(1ms); }
  }
```

**The abstraction is agnostic to all of these choices.**

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

Examples:
- Protocol: Generic (static dispatch, performance)
- Storage/Cache: Dynamic (swap SQLite ↔ RocksDB at construction)
- Transport: Concrete type (TCP, maximum performance)
- Timer: Concrete (no swapping needed)
- ExternalInterface: Dynamic (swap HTTP ↔ gRPC ↔ WebSocket)

This creates a **flexibility/performance tradeoff at the dimension level**, not system level.

### Mixed Strategies in Production

Real implementations commonly mix strategies:
```
StaticProtocolNode:
  protocol: RaftProtocol              (static - zero cost)
  storage: Box<dyn Storage>           (dynamic - test vs prod)
  cache: Box<dyn Cache>               (dynamic)
  transport: TcpTransport             (concrete - performance)
  external_interface: Box<dyn ExternalInterface> (dynamic - HTTP vs gRPC)
  timer: RealtimeTimer                (concrete)
```

---

## Design Decisions

### 1. No Separate Executor

Early designs had a seventh dimension: "Executor."

**Analysis revealed:** With maximal abstraction, the executor collapses into the Node itself. The `step()` method IS the execution primitive. Event loops, async wrappers, and scheduling are implementation details, not core abstractions.

**Six dimensions, not seven.**

### 2. Implicit Dimension Access

The `step()` method does not take dimensions as parameters:
```rust
fn step(&mut self) -> bool;  // ✅ Correct
```

Not:
```rust
fn step(&mut self, protocol: &P, storage: &mut S, ...) -> bool;  // ❌ Wrong
```

**Rationale:** This would prescribe internal organization. The trait defines capabilities, not structure. Implementations might:
- Store dimensions as struct fields
- Compute them on-demand
- Proxy to remote services
- Use enum variants
- Share dimensions across nodes (Arc)

### 3. Associated Types with Trait Bounds

Each dimension's associated type is constrained by its infrastructure trait:
```rust
type Storage: Storage<Query = Self::StorageQuery, ...>;
```

This ensures:
- Type safety at the trait level
- Protocol-specific types flow through the system
- Compile-time verification of compatibility

### 4. Protocol-Specific Types

Storage, Cache, Transport, ExternalInterface, and Timer all use protocol-specific associated types:
- `StorageQuery`, `StorageMutation` — Defined by protocol needs
- `CacheQuery`, `CacheUpdate` — Algorithm-specific state
- `PeerMessage` — Protocol messages
- `ClientRequest`, `ClientResponse` — API-specific
- `TimerEvent` — Protocol timeouts/scheduling

**No generic event types.** Each protocol defines exactly what it needs.

---

## Abstraction Layers

### Layer 1: Infrastructure Traits (Core)

Seven traits defining capabilities:
- `ConsensusProtocol` — Stateless protocol logic
- `Storage` — Persistent state operations
- `Cache` — Volatile state operations
- `Transport` — Peer communication
- `ExternalInterface` — Client communication
- `Timer` — Time-based events
- `Collection` — Generic sequence abstraction

**Responsibility:** Define interfaces, not implementations.

### Layer 2: Node Trait (Core)

Single trait composing the six dimensions:
- Associated types for each dimension
- Trait bounds connecting to Layer 1
- `step()` execution primitive

**Responsibility:** Define the node contract.

### Layer 3: Implementations (Outside Core)

Concrete types implementing the traits:
- `RaftNode<P>` — Static protocol, dynamic infrastructure
- `DynamicNode` — All dynamic (for Torque testing)
- `StaticNode<P, S, C, T, E, Ti>` — Fully generic (zero cost)
- Protocol implementations (Raft, PBFT, etc.)
- Infrastructure implementations (InMemory, SQLite, TCP, etc.)

**Responsibility:** Realize the abstractions with specific strategies.

---

## Validation Philosophy

The architecture enables:

### Deterministic Testing
- `step()` primitive enables event-by-event control
- In-memory infrastructure implementations
- Deterministic time (controlled timer advancement)
- No async/await in core traits (synchronous poll-based)

### Exhaustive Exploration
- Stateless protocols enable property-based testing
- Small state spaces can be fully explored
- Failure injection through controlled infrastructure

### Cross-Environment Deployment
- Same core logic runs in:
  - In-memory simulation (tests)
  - Production (tokio, async)
  - Embedded (Embassy, no-std)
  - Custom environments (future)

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
├─ Any Storage (trait-based)
├─ Any Transport (trait-based)
├─ Any Protocol (trait-based)
├─ Any Cache (trait-based)
├─ Any ExternalInterface (trait-based)
└─ Any Timer (trait-based)
```

**Benefits:**
- Test protocols with in-memory infrastructure
- Swap components at compile-time or runtime
- Validation through deterministic simulation
- One abstraction, many implementations

---

## Future Generalizations

### SystemEntity (Beyond Blockchain)

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
- Raft nodes
- UAV swarms
- Autonomous vehicles
- Distributed controllers
- Robot coordination

The six-dimensional decomposition generalizes beyond blockchain to any distributed system participant.

---

## Key Takeaways

1. **Maximal Abstraction** — Core defines contracts (traits), not implementations
2. **Six Dimensions** — Protocol, Storage, Cache, Transport, ExternalInterface, Timer
3. **One Primitive** — `step()` is the only required method
4. **Flexibility Spectrum** — Each dimension independently chooses static/dynamic/concrete
5. **Implicit Structure** — Trait doesn't prescribe how dimensions are organized
6. **Protocol-Specific Types** — Infrastructure uses protocol-defined types, not generic events
7. **Implementation Freedom** — Radical variation in implementation strategy without changing core

This architecture achieves the project's core goal:

> Separate what must be correct from what may vary.
