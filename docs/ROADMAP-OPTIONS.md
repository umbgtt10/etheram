# EtheRAM — Roadmap Options

This document catalogues all candidate directions for the project's next phase, organised into seven categories (A–G). Each option includes effort estimate and architectural rationale.

---

## Direction A: More Raft

| # | Feature | Effort | Value |
|---|---|---|---|
| A1 | **Raft configuration changes (joint consensus)** — `AddServer` / `RemoveServer` with the two-phase joint-consensus protocol from the Raft extended paper | High | Completes the Raft spec; proves the model handles membership changes (a notoriously tricky area) |
| A2 | **Raft read leases** — `ReadIndex` and/or lease-based reads that avoid full log replication for read-only queries | Medium | Practically important; exercises the Client→Protocol→Response path more deeply |
| A3 | **Raft log compaction with real snapshots** — currently `SaveSnapshot` and `InstallSnapshot` exist but snapshots are in-memory byte blobs. Add a `SnapshotStorage` dimension that serializes/deserializes to semihosting files | Medium | Tests the decomposition's extensibility — adding a 7th dimension or sub-dimension |
| A4 | **Raft embassy multi-node fault injection** — extend the 5-act QEMU scenario to kill a node mid-replication and verify the cluster continues (3-of-5 survival) | Low | Validates no_std fault tolerance at the embedded layer |

---

## Direction B: More IBFT

| # | Feature | Effort | Value |
|---|---|---|---|
| B1 | **IBFT round-change certificate aggregation** — currently view changes work but don't carry aggregated signatures (BLS). Add a `BlsSignatureScheme` behind the existing `SignatureScheme` trait and aggregate prepare signatures into a single proof | High | Architecturally interesting (tests swappability of crypto); practically necessary for real BFT systems |
| B2 | **IBFT Byzantine fault injection via builder** — message injection exists in cluster tests; the missing piece is a set of adversarial `ProtocolVariant` implementations (equivocating proposer, double-voting, message-withholding) that can be swapped in via `IbftNodeBuilder` without test-level orchestration | Low | Completes the "Testing as Muscle" principle — swap-based adversarial testing, not scenario scripting |
| B3 | **IBFT finality gadget** — implement a finality observer that tracks which blocks are irreversible (2f+1 commits received) and exposes `latest_finalized_height` | Low | Useful for the EVM execution path; demonstrates the Observer being used for non-logging purposes |

---

## Direction C: More Ethereum-like Functionalities

| # | Feature | Effort | Value |
|---|---|---|---|
| C1 | **Expand TinyEVM** — add `MSTORE`/`MLOAD` (memory model), `CALLDATALOAD`/`CALLDATASIZE`, `SHA3`, `JUMP`/`JUMPI`. This would make TinyEVM capable of running simple Solidity-compiled contracts | High | Moves TinyEVM from toy to credible subset; proves the ExecutionEngine trait scales |
| C2 | **Merkle Patricia Trie state root** — replace the current XOR-mix hash with a real MPT (or a simplified binary trie) for `compute_state_root`. Ethereum clients use this for state proofs | High | Architecturally significant — tests whether storage abstraction can accommodate proof-generating state stores |
| C3 | **Transaction pool with priority ordering** — currently pending transactions are a flat `Vec`. Add gas-price-based ordering, per-sender nonce sequencing, and a configurable pool size limit | Medium | Practical and exercises the Cache dimension more substantially |
| C4 | **JSON-RPC external interface** — implement a subset of the Ethereum JSON-RPC spec (`eth_sendTransaction`, `eth_getBalance`, `eth_blockNumber`, `eth_getTransactionReceipt`) as an `ExternalInterface` variant | Medium | Makes the node queryable by standard Ethereum tooling; validates ExternalInterface swappability with a real protocol |
| C5 | **Contract deployment** — add `CREATE` opcode and contract account storage so that TinyEVM can deploy and call contracts (even trivially) | High | The single biggest step toward Ethereum-likeness; requires memory model + address derivation |
| C6 | **Block gas limit enforcement** — currently `MAX_GAS_LIMIT` is per-transaction. Add a block-level gas limit so that the proposer fills blocks up to the limit, ordering by gas price | Low-Medium | Straightforward extension of gas metering; exercises proposer logic |

---

## Direction D: Prove Generality — A Third Protocol Family

The strongest claim of the 3-6 model is that it generalizes. Two protocols (BFT + CFT) are suggestive; three would be compelling.

| # | Option | Effort | Why |
|---|---|---|---|
| D1 | **HotStuff** — linear-communication BFT with pipelined phases (used by Meta's Diem/Aptos) | High | Different phase structure from IBFT (3-chain vs 4-phase); tests whether the `step()` pipeline accommodates pipelining |
| D2 | **Multi-Paxos** — classic CFT with distinguished proposer, acceptors, learners | Medium | Structurally different from Raft (separate roles vs unified replicas); reveals whether the decomposition assumes Raft-shaped state |
| D3 | **Tendermint** — BFT with propose/prevote/precommit and block-level finality | Medium | Close enough to IBFT to reuse infrastructure, different enough (no view-change message, timeout-driven rounds) to stress the model |

A third family in its own crate triplet (e.g. `hotstuff-node/` + `hotstuff-validation/` + `hotstuff-embassy/`) — depending only on `core/` — would be the single most convincing proof that the decomposition is general, not accidental.

---

## Direction E: Formal Methods — Verify the Pure Protocol Core

The fact that `handle_message()` is pure (immutable context in, declarative actions out, zero I/O) makes the protocol logic an ideal target for formal verification. No other Rust consensus implementation has this property at the architectural level.

| # | Option | Effort | Why |
|---|---|---|---|
| E1 ✅ | **TLA+ specifications** — write TLA+ models of IBFT and Raft that mirror the Rust protocol handlers, then model-check safety (no two blocks committed at same height) and liveness (progress under <f failures) | Medium | Verifies the protocol logic independently of Rust; publishable artifact |
| E2 | **Kani model checking** — use [Kani](https://github.com/model-checking/kani) (Rust-native bounded model checker by AWS) to exhaustively verify `handle_message()` invariants: e.g., two honest nodes never emit conflicting `StoreBlock` actions at the same height | Medium | Runs directly on the Rust source — no translation layer. Leverages the purity that the architecture enforces |

---

## Direction F: Production Infrastructure Layer

The decomposition is validated in-memory and on QEMU. A production infrastructure layer would test whether the trait boundaries hold under real system pressures (latency, disk I/O, backpressure).

| # | Option | Effort | Why |
|---|---|---|---|
| F1 | **libp2p transport** — implement `TransportIncoming` + `TransportOutgoing` over libp2p (GossipSub for broadcasts, request-response for point-to-point) | High | This is what real Ethereum clients use; validates the transport abstraction against real network semantics (peer discovery, NAT traversal, multiplexing) |
| F2 | **RocksDB storage** — implement `Storage` over RocksDB with column families mapping to the query/mutation types | Medium | Tests whether the storage abstraction handles batch writes, compaction, and crash recovery without leaking RocksDB concepts into the protocol |
| F3 | **Docker cluster deployment** — 5 nodes in containers, real TCP, scripted fault injection (kill/partition containers) | Medium | End-to-end integration beyond QEMU; demonstrates the framework produces deployable nodes |
| F4 | **Throughput benchmarks** — measure transactions/second and consensus rounds/second, profile trait-object dispatch overhead vs monomorphized implementations | Low-Medium | Answers the practical question: does the decomposition's flexibility cost performance? |

---

## Direction G: Tooling and Observability

The `Observer` trait already instruments every step. Building on it would showcase the decomposition as a *debuggable* architecture, not just a testable one.

| # | Option | Effort | Why |
|---|---|---|---|
| G1 | **Protocol state visualizer** — a web dashboard (via a WebSocket `ExternalInterface` variant) that shows live round/term, vote counts, block height, and message flow between nodes | Medium | Demonstrates ExternalInterface swappability for a non-trivial use case; makes the project visually demonstrable |
| G2 | **Deterministic replay debugger** — record all `(source, message)` inputs to a node, then replay them step-by-step with full observer output. The pure protocol makes this trivially correct | Low-Medium | Unique capability enabled by the architecture; no other consensus framework offers guaranteed-correct replay by construction |
| G3 | **Chaos testing framework** — a `ChaosTransport` wrapper that randomly drops, delays, reorders, or duplicates messages. Compose it with any real transport via decoration | Low | Directly validates the "Testing as Muscle" principle at the transport dimension level |

---

## Recommended Sequencing

### Maximise architectural validation (moderate effort)
A4 → B2 → C3 → C6

### Deepen the Ethereum-like story
C1 → C3 → C6 → C5

### Complete the consensus story
A1 → A2 → B1

### One pick from each direction
- **D2 (Multi-Paxos)** — fastest path to proving generality across three consensus families
- **E2 (Kani)** — highest research impact, directly leverages the purity guarantee
- **F4 (Benchmarks)** — cheapest way to answer the "does this scale" question
- **G2 (Deterministic replay)** — unique to this architecture, low effort, high demo value
