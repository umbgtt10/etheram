# ADR-001: Six-Dimension Node Decomposition

## Status

**Accepted** - Validated by TinyChain implementation (January 2026)

## Context

Traditional blockchain node implementations tightly couple consensus logic with infrastructure concerns (networking, persistence, time). This creates several problems:

1. **Testing Difficulty** - Cannot validate consensus without real networks, storage, or clocks
2. **Implementation Lock-in** - Changing storage or transport requires rewriting protocol logic
3. **Unclear Boundaries** - No explicit contracts between system components
4. **Limited Reusability** - Cannot swap protocols or infrastructure independently

We need a decomposition that:
- Separates protocol semantics from infrastructure
- Enables independent testing of each concern
- Allows free swapping of implementations
- Maintains clear, explicit boundaries

## Decision

We decompose a blockchain node into **six orthogonal dimensions**:

### 1. Protocol
Consensus algorithm implemented as stateless, pure functions. Takes context and events, returns actions. Contains no I/O, no state mutation, only logic.

### 2. Storage
Persistent state that survives crashes. Handles chain data, account balances, committed blocks. Provides query/mutation interface.

### 3. Cache
Volatile working state used by the algorithm. Examples: pending transactions, leader role, peer connectivity. Lost on restart.

### 4. Transport
Peer-to-peer message delivery. Sends/receives protocol messages to/from other nodes. Abstracts network topology.

### 5. ExternalInterface
Client request/response handling. Receives transactions, queries, returns responses. Separates peer and client communication.

### 6. Timer
Time-based event scheduling. Triggers periodic actions (block proposals, heartbeats, timeouts). Abstracts time source.

**Key Principles:**
- Each dimension has a single, well-defined responsibility
- Dimensions interact only through the Node orchestration layer
- Protocol logic is independent of infrastructure choices
- All dimensions are independently swappable

## Consequences

### Positive

1. **Clean Separation** - Each dimension has explicit contracts and boundaries
2. **Independent Testing** - Can test protocol with mock storage, mock transport, mock timer
3. **Free Swapping** - Replace any dimension without touching others
4. **Determinism** - In-memory implementations enable deterministic testing
5. **Protocol Reuse** - Same protocol works with different infrastructure
6. **Infrastructure Reuse** - Same storage/transport works with different protocols

### Negative

1. **Indirection Overhead** - More trait calls vs monolithic design (minimal in practice)
2. **Design Complexity** - Requires upfront thinking about boundaries
3. **Type Verbosity** - Each dimension needs associated types defined

### Validation Evidence

TinyChain implementation demonstrates:
- ✅ Complete functional decomposition with no dimension leakage
- ✅ 20 deterministic tests using in-memory dimensions
- ✅ Protocol logic independent of infrastructure
- ✅ Zero mutable getters - true encapsulation achieved
- ✅ Clean swapping path for any dimension

## Related

- [ADR-002: step() as Single Execution Primitive](002-step-as-single-execution-primitive.md)
- [Architecture Documentation](../ARCHITECTURE.md)
- TinyChain implementation: `examples/tinychain/`
