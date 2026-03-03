# The 3-6 Model for Blockchain Architecture

**Date:** January 26, 2026
**Status:** Discovery Phase - To be consolidated

## Overview

The 3-6 Model is an architectural pattern for blockchain nodes that emerged from principled design decisions focused on separation of concerns, testability, and operating-model independence. It consists of **3 layers** and **6 dimensions** that together form a complete, clean blockchain node architecture.

## The Three Layers

### 1. Protocol Layer (The Brain)
**Responsibility:** Pure decision logic

**Characteristics:**
- Stateless (`&self` only)
- Pure function: `fn handle_message(source, message, context) -> actions`
- No I/O, no mutations, no infrastructure dependencies
- Operating-model agnostic (works with any execution environment)

**Benefits:**
- Fully testable with simple function calls
- Reusable across different node implementations
- Swappable consensus algorithms
- Formal verification feasible

**Example:**
```rust
impl ConsensusProtocol for TinyChainProtocol {
    fn handle_message(
        &self,
        source: MessageSource,
        message: Self::Message,
        ctx: &Self::Context,
    ) -> Self::ActionCollection {
        // Pure computation - no side effects
    }
}
```

### 2. State Layer (The Memory)
**Responsibility:** Dimension management (read + write)

**Components:**
- **Context Builder**: Reads from Storage & Cache → produces immutable Context
- **Action Executor**: Takes Actions → writes to Storage & Cache

**Characteristics:**
- Owns Storage and Cache dimensions
- Read-only interface for Protocol (via Context)
- Read-write for execution
- Can be transactional and atomic

**Benefits:**
- Unified state management
- Swappable backends (Memory, Persistent, Distributed)
- Transactional semantics possible
- Clear ownership model

**Future abstraction:**
```rust
trait DimensionManager {
    fn build_context(&self) -> Context;  // Read
    fn execute_actions(&mut self, actions: ActionCollection<Action>);  // Write
}
```

### 3. Coordination Layer (The Nervous System)
**Responsibility:** I/O routing and orchestration

**Characteristics:**
- Polls I/O dimensions (Transport, ExternalInterface, Timer)
- Routes messages to Protocol
- Executes returned actions via State Layer
- Defines operating model (async, sync, actor, thread)
- **Zero business logic**

**Benefits:**
- Operating model flexibility
- Simple, obviously correct code
- Easy to reason about concurrency
- Minimal state

**Pattern:**
```rust
pub fn step(&mut self) -> bool {
    // Poll dimension → Build context → Route to protocol → Execute actions
    if let Some((peer_id, peer_message)) = self.transport.poll() {
        let context = self.dimension_mgr.build_context();
        let message = Message::Peer(peer_message);
        let actions = self.protocol.handle_message(source, message, &context);
        self.dimension_mgr.execute_actions(actions);
        return true;
    }
    // ... repeat for client and timer
}
```

## The Six Dimensions

### State Dimensions (Owned by State Layer)
1. **Storage** - Persistent blockchain state (blocks, balances, etc.)
2. **Cache** - Ephemeral state (mempool, pending transactions)

### I/O Dimensions (Owned by Coordination Layer)
3. **Transport** - Peer-to-peer communication
4. **External Interface** - Client communication (RPC, API)
5. **Timer** - Temporal events and scheduling

### Logic Dimension
6. **Protocol** - Decision engine (consensus algorithm)

## The Complete Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    COORDINATION LAYER                        │
│  (Node - I/O Routing, Operating Model)                      │
│                                                              │
│  Poll 3 sources: Transport, ExternalInterface, Timer        │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   STATE LAYER          │
              │   (Dimension Manager)  │
              │                        │
              │   build_context()      │
              │   ↓                    │
              │   Read: Storage, Cache │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   PROTOCOL LAYER       │
              │   (The Brain)          │
              │                        │
              │   handle_message()     │
              │   Context → Actions    │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   STATE LAYER          │
              │   (Dimension Manager)  │
              │                        │
              │   execute_actions()    │
              │   ↓                    │
              │   Write: Storage, Cache│
              │   Write: Transport, etc│
              └────────────────────────┘
```

## Key Architectural Decisions

### 1. Unified Message Handling
**Decision:** Protocol handles ALL inputs (peer, client, timer) through single entry point

**Traditional approach:** Separate handlers for consensus vs client vs timer
- Consensus module handles peer messages
- Mempool handles client transactions
- Block production module handles timers
- Coordination between modules via shared state and locks

**Our approach:** Single `handle_message(source, message, context)` entry point
```rust
enum Message {
    Peer(PeerMessage),
    Client(ClientRequest),
    Timer(TimerEvent),
}

enum MessageSource {
    Peer(PeerId),
    Client(ClientId),
    Timer,
}
```

**Why it's better:**
- No artificial separation between "consensus" and "other concerns"
- Protocol can correlate events across dimensions
- No race conditions from concurrent module access
- Simpler testing (just call one function)
- Semantically honest (all three ARE consensus decisions)

**Traditional blockchain philosophy challenged:**
Most blockchains separate consensus from client/timer handling based on implementation constraints (threading models, historical architecture), not sound design principles. The unified approach is architecturally superior:
- Eliminates coordination complexity
- Protocol-specific logic stays in protocol
- No shared mutable state between "components"
- Sequential message processing = clear semantics

### 2. Context Ownership
**Decision:** Context owns all data (no references/lifetimes)

**Rationale:**
- ConsensusProtocol trait cannot have lifetime parameters on associated types
- Immutable snapshots enable pure functional reasoning
- Built fresh for each message by cloning from dimensions
- Protocol cannot accidentally observe mutations mid-execution

**Pattern (from tinychain):**
```rust
#[derive(Debug, Clone)]
pub struct Context {
    pub node_id: NodeId,
    pub current_height: Height,
    pub pending_txs: Vec<Transaction>,
    pub is_leader: bool,
    pub accounts: HashMap<Address, Balance>,
}
```

### 3. Actions as Pure Data
**Decision:** Protocol returns ActionCollection, execution is separate

**Benefits:**
- Protocol can be tested without executing actions
- Actions can be logged, queued, batched, or optimized
- Execution strategy is swappable
- Clear separation: decide vs execute

**Action Executor abstraction (future):**
```rust
trait ActionExecutor {
    fn execute(&mut self, actions: ActionCollection<Action>);
}
```
Enables: batching, async execution, transactional execution, distributed execution

### 4. Protocol as Pure Computation
**Decision:** Protocol is stateless (`&self`), no internal mutation

**Implications:**
- Testing: `protocol(context, message) == expected_actions` - just assert equality
- Property-based testing: Generate random (context, message) pairs
- Formal verification: Pure state machine, TLA+ model maps directly
- No concurrency issues within protocol
- Reusable across different operating models

**The killer insight:** Protocol = static algorithm with no infrastructure dependency. Testing becomes muscle work, not creative reasoning.

## Comparison with Traditional Architecture

### Traditional Blockchain Node
```
┌──────────────────────────────────────────┐
│  Consensus Module (with locks)           │
│    ↕                                     │
│  Mempool Module (with locks)            │
│    ↕                                     │
│  Block Production (with timers/threads)  │
│    ↕                                     │
│  Shared State (database, memory)        │
└──────────────────────────────────────────┘
```
**Problems:**
- Race conditions between modules
- Coordination via locks/channels
- Business logic scattered across modules
- Testing requires integration tests
- Operating model baked into design

### 3-6 Model
```
┌────────────────────────────────────────┐
│  Node (router only)                    │
│    ↓                                   │
│  DimensionManager (state)              │
│    ↓                                   │
│  Protocol (pure function)              │
│    ↓                                   │
│  DimensionManager (execution)          │
└────────────────────────────────────────┘
```
**Advantages:**
- No race conditions (sequential message processing)
- Single source of truth for decisions
- Unit testable protocol
- Operating model independent
- Clear ownership and responsibilities

## Benefits

### Testability
- **Protocol:** Pure function testing, property-based testing, fuzzing
- **State Layer:** Mock dimensions, verify state transitions
- **Coordination:** Test I/O routing without business logic

### Flexibility
- **Swap execution strategies:** Memory → Persistent → Distributed
- **Swap operating models:** Sync → Async → Actor → Thread-per-dimension
- **Swap protocols:** Drop-in different consensus algorithms
- **Swap backends:** Different storage/cache implementations

### Correctness
- **Protocol:** Formal verification feasible (pure state machine)
- **State Layer:** Transactional semantics possible
- **Coordination:** Simple enough to be obviously correct

### Performance
- **Action batching:** Execute multiple actions atomically
- **Parallel execution:** Different action types can parallelize
- **State caching:** DimensionManager can cache context building
- **Async I/O:** Coordination layer handles async without protocol changes

## Implementation Status

### Completed
- ✅ Core ConsensusProtocol trait with unified `handle_message()`
- ✅ MessageSource enum (Peer, Client, Timer)
- ✅ Context owns data (no lifetimes)
- ✅ ActionCollection pattern
- ✅ Protocol as pure function (`&self` only)
- ✅ Tinychain reference implementation
- ✅ Embassy async implementation

### Current State
- Protocol Layer: ✅ Complete
- State Layer: ⚠️ Embedded in Node (needs extraction to DimensionManager)
- Coordination Layer: ✅ Clean routing pattern

### Future Refactoring
1. Extract `DimensionManager` trait and implementation
2. Move `build_context()` and `execute_actions()` to DimensionManager
3. Node becomes ultra-thin router
4. Multiple DimensionManager implementations (Memory, Persistent, etc.)

## Testing Implications

### Protocol Testing (Pure Function)
```rust
#[test]
fn test_block_validation() {
    let protocol = TinyChainProtocol::new(node_count);
    let context = Context { height: 5, ... };
    let message = Message::Peer(PeerMessage::ProposeBlock(block));

    let actions = protocol.handle_message(
        MessageSource::Peer(peer_id),
        message,
        &context
    );

    // Just assert expected actions - no mocking!
    assert_eq!(actions.len(), 3);
    assert!(matches!(actions[0], Action::AcceptBlock(_)));
}
```

### Property-Based Testing
```rust
proptest! {
    fn protocol_never_double_spends(
        context: Context,
        message: Message
    ) {
        let actions = protocol.handle_message(source, message, &context);
        assert!(no_double_spend(actions));
    }
}
```

### Systematic Testing = Muscle, Not Brain
With protocol as pure computation:
- Exhaustive state exploration: N nodes × M messages = NM function calls
- No infrastructure setup needed
- Fuzzing directly applicable
- Formal verification maps 1:1 to implementation

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

### 4. Pipelined Processing (for EtheRAM)
```rust
// While executing block N, verify block N+1, receive block N+2
struct PipelinedNode {
    verification_queue: Channel<Block>,
    execution_queue: Channel<ActionCollection>,
}
```

## Why This Emerged

**We didn't design this upfront.** It emerged from asking:

1. "Should protocol be testable?" → Pure function
2. "Should protocol work with any runtime?" → No async/threads in protocol
3. "Should we separate consensus from client handling?" → NO - it's all consensus decisions
4. "Can protocol have mutable state?" → NO - makes testing hard
5. "Should context have references?" → NO - trait lifetime constraints

**Each decision forced the next**, and the 3-6 model appeared as the natural consequence of principled choices.

## Validation Strategy

**The architecture will be truly validated with EtheRAM implementation:**
- Complex cryptographic verification
- EVM execution engine integration
- Higher performance requirements
- More sophisticated state management needs

If the 3-6 model scales to EtheRAM's complexity while maintaining simplicity at the Node level, it confirms the architecture is sound.

## Open Questions

1. **Context building cost:** Cloning all data for each message - is this prohibitive at scale?
   - Possible mitigation: Arc/Rc for immutable shared data
   - Possible mitigation: Copy-on-write data structures

2. **Action granularity:** Should actions be fine-grained or coarse-grained?
   - Fine: More flexibility for execution optimization
   - Coarse: Less protocol/executor coupling

3. **Error handling:** How should protocol signal errors?
   - Return Result<ActionCollection, ProtocolError>?
   - Include Error action type?

4. **Async protocol?** Should handle_message be async?
   - Current: Synchronous (pure computation)
   - Future: Might need async for cryptographic operations?

## Related Documentation

- [ConsensusProtocol Trait Definition](../core/src/consensus_protocol.rs)
- [Tinychain Reference Implementation](../examples/tinychain/)
- [Embassy Async Implementation](../embassy/)

## Revision History

- 2026-01-26: Initial discovery and documentation during Embassy refactoring

---

**Note:** This document captures our architectural discoveries. It will be consolidated and restructured after EtheRAM implementation validates the approach.
