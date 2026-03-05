# ADR-001: Six-Dimension Node Decomposition

## Status

**Accepted** — Validated by Etheram and Raft implementations (January–March 2026)

## Context

Traditional blockchain node implementations tightly couple consensus logic with infrastructure concerns (networking, persistence, time). This creates several problems:

1. **Testing Difficulty** — Cannot validate consensus without real networks, storage, or clocks
2. **Implementation Lock-in** — Changing storage or transport requires rewriting protocol logic
3. **Unclear Boundaries** — No explicit contracts between system components
4. **Limited Reusability** — Cannot swap protocols or infrastructure independently

We need a decomposition that:
- Separates protocol semantics from infrastructure
- Enables independent testing of each concern
- Allows free swapping of implementations
- Maintains clear, explicit boundaries

## Decision

We decompose a blockchain node into **six orthogonal dimensions**:

### 1. Protocol
Consensus algorithm implemented as stateless, pure functions. Takes context and events, returns declarative actions. Contains no I/O, no state mutation, only logic. Etheram validates this with `IbftProtocol<S>`, a full IBFT BFT consensus implementation that is entirely pure — `handle_message()` takes an immutable context and returns an action collection.

### 2. Storage
Persistent state that survives crashes. Handles chain data, account balances, committed blocks, transaction receipts, and contract storage. Provides query/mutation interface. Etheram provides `InMemoryStorage` (std, deterministic testing) and `SemihostingStorage` (no_std, ARM semihosting file I/O). Both auto-recompute a deterministic state root on every `UpdateAccount` mutation.

### 3. Cache
Volatile working state used by the algorithm. Examples: pending transactions, vote tracking, prepared certificates, current round. Lost on restart. Etheram uses `InMemoryCache` for pending transaction management, with the IBFT protocol maintaining its own internal consensus state (vote trackers, round, locked block).

### 4. Transport
Peer-to-peer message delivery. Sends/receives protocol messages to/from other nodes. Abstracts network topology. Etheram provides `InMemoryTransport` (shared `Arc<Mutex>` state for deterministic cluster testing), `OutboxTransport` (sync-to-async bridge for Embassy channel routing), and `UdpIbftTransport` (postcard-serialized UDP message passing via `WireIbftMessage` mirror types).

### 5. ExternalInterface
Client request/response handling. Receives transactions, queries (balance, height, nonce, state root), returns responses. Separates peer and client communication. Etheram supports `InMemoryExternalInterface`, channel-based (`ClientChannelHub`), and UDP-backed external interfaces.

### 6. Timer
Time-based event scheduling. Triggers block proposals (`ProposeBlock`), round timeouts (`TimeoutRound`), and heartbeats. Abstracts time source. Etheram uses `InMemoryTimer` (with `push_event` for deterministic test driving) and Embassy async timer receivers for embedded deployment.

**Key Principles:**
- Each dimension has a single, well-defined responsibility
- Dimensions interact only through the Node orchestration layer (`EtheramNode.step()`)
- Protocol logic is independent of infrastructure choices
- All dimensions are independently swappable via `Box<dyn Trait>` trait objects

## Consequences

### Positive

1. **Clean Separation** — Each dimension has explicit contracts and boundaries; the Partitioner enforces side-effect isolation by separating mutations from outputs from execution actions
2. **Independent Testing** — Protocol tested with mock storage, mock transport, mock timer across 557 deterministic tests
3. **Free Swapping** — Replace any dimension without touching others; dimensions are selected at construction via builders (`StorageBuilder`, `ProtocolBuilder`, `EtheramNodeBuilder`)
4. **Determinism** — In-memory implementations enable fully deterministic testing with explicit event injection
5. **Protocol Reuse** — Same `IbftProtocol<S>` works with in-memory infrastructure (testing), Embassy channels (embedded), and UDP/semihosting (real hardware)
6. **Infrastructure Reuse** — Same `InMemoryStorage` works with `IbftProtocol`, `NoOpProtocol`, or any future protocol implementation
7. **Observability** — The `Observer` trait provides per-step hooks (`action_emitted`, `mutation_applied`, `output_executed`) giving full visibility into every dimension interaction without silent side-effects

### Negative

1. **Indirection Overhead** — More trait calls (vtable dispatch via `Box<dyn Trait>`) vs monolithic design (minimal in practice; Embassy QEMU execution validates acceptable overhead on Cortex-M4)
2. **Design Complexity** — Requires upfront thinking about boundaries and partitioning of actions into mutations/outputs/executions
3. **Type Verbosity** — Each dimension needs associated types defined; mitigated by builder patterns in `etheram-node` and `raft-node`

### Validation Evidence

Etheram validates the six-dimension decomposition across three deployment environments:

**Stage 1 — Unit and single-node tests (etheram-node):**
- 557 deterministic tests using in-memory dimensions, zero external dependencies
- Complete IBFT BFT consensus: pre-prepare, prepare, commit, view change, validator set updates, future-round buffering, deduplication, replay protection, WAL persistence
- Two execution engines (`ValueTransferEngine`, `TinyEvmEngine`) demonstrating Engine swappability
- Ed25519 and Mock signature schemes demonstrating cryptographic swappability
- Pure protocol logic: `IbftProtocol::handle_message()` takes immutable context, returns declarative actions — no I/O anywhere in protocol code

**Stage 2 — Cluster tests (etheram-validation):**
- `IbftCluster` orchestrates multiple `EtheramNode<IbftMessage>` instances with shared in-memory transport/timer/EI state
- Byzantine fault injection, malicious block detection, message validation, round progression, persistence, gas metering
- 130+ cluster-level integration tests validating distributed correctness
- All tests execute deterministically via explicit `step()` calls with controlled event ordering

**Stage 3 — Embedded deployment (etheram-embassy, QEMU):**
- Same `EtheramNode` and `IbftProtocol` running on `no_std` ARM Cortex-M4 target
- Two independently maintained configurations: all-in-memory (channel transport + in-memory storage + channel EI) and real (UDP transport + semihosting storage + UDP EI)
- 12-act scenario exercising transfers, overdraft rejection, view changes, stale nonce rejection, gas limits, validator set updates, WAL round-trip, Ed25519 signatures, TinyEVM contract storage, and OutOfGas revert
- Cross-environment proof: identical `etheram-node` logic compiles and executes correctly across std (testing), std (cluster validation), and no_std (ARM embedded)

Raft independently validates the same six-dimension decomposition across the mirrored crate family:

**Stage 1 — Protocol-level tests (raft-node):**
- Pure `RaftProtocol<P>` logic tested for election, replication, snapshots, client handling, and role transitions
- Uses the same decision/infrastructure split as Etheram, with protocol-specific Raft types

**Stage 2 — Cluster tests (raft-validation):**
- `RaftCluster` orchestrates multiple `RaftNode<P>` instances through deterministic `step()` execution
- Validates distributed correctness under failures, re-elections, snapshot flow, and state-machine apply

**Stage 3 — Embedded deployment (raft-embassy, QEMU):**
- Same `RaftNode<P>` runs on no_std ARM Cortex-M4 under Embassy async runtime
- Two independently maintained configurations validated end-to-end: all-in-memory and UDP+semihosting
- 5-act scenario validated in QEMU: election, replication, read-after-write, re-election, continued replication

Together, Etheram and Raft confirm that the six-dimension node decomposition is architectural, not protocol-specific.

## Related

- [ADR-002: step() as Single Execution Primitive](002-step-as-single-execution-primitive.md)
- [Architecture Documentation](../ARCHITECTURE.md)
