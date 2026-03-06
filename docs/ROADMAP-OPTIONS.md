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

## Direction H: Desktop Application — Multi-Node gRPC Cluster

Run a real multi-node EtheRAM cluster as a native desktop application: each node in
its own OS process, transport over gRPC, state persisted in an embedded database.
This direction proves the decomposition holds under real system pressures without
requiring dedicated server hardware or container orchestration.

| # | Feature | Effort | Value |
|---|---|---|---|
| H1 | **gRPC transport** — implement `TransportIncoming` + `TransportOutgoing` over [tonic](https://github.com/hyperium/tonic) (gRPC over HTTP/2). Point-to-point `SendMessage` maps to unary RPC; `BroadcastMessage` fans out concurrently. A protobuf codec replaces `WireIbftMessage` | Medium | Validates transport swappability against a real, widely-used RPC protocol; natural fit for cross-language clients later |
| H2 | **Embedded database storage** — implement `StorageAdapter` over [sled](https://github.com/spacejam/sled) (pure-Rust embedded B-tree, no external process). Each storage mutation becomes a transactional sled batch | Medium | Tests whether the storage abstraction accommodates crash-safe, concurrent write semantics without leaking sled concepts into the protocol |
| H3 | **Per-node process harness** — an `etheram-node-process` binary crate that runs a single `EtheramNode` step loop, wired with gRPC transport + sled storage, driven by a `cluster.toml` entry passed as a CLI argument. A separate `etheram-desktop` launcher process reads the fleet config and spawns one `etheram-node-process` child per node, monitoring their stdout/stderr for the dashboard UI | Medium | End-to-end proof that the decomposition composes under real OS process boundaries — full fault isolation, independent address spaces, genuine crash recovery |
| H4 | **Desktop UI — terminal dashboard** — a [ratatui](https://github.com/ratatui-org/ratatui) terminal UI showing live per-node height, round, role, pending tx count, and last committed block hash, updated via the `Observer` trait | Medium | Makes the cluster visually observable; demonstrates that the Observer dimension serves non-logging purposes |
| H5 | **gRPC `ExternalInterface`** — a tonic-backed `ExternalInterfaceIncoming` + `ExternalInterfaceOutgoing` pair exposing `SubmitTransaction`, `GetBalance`, `GetHeight`, and `GetBlock` as unary RPCs. Separate from the peer transport (H1); client-facing and peer-facing gRPC services run on different ports | Medium | Completes the gRPC story — without this there is no way for a user or external tool to talk to a running desktop node |
| H6 | **WAL-backed crash recovery** — wire `ConsensusWal` + a real `WalWriter` implementation (writing to sled or a flat append-only file) so that a restarted node recovers its `prepared_certificate`, current round, and locked block before rejoining the cluster. `NoOpWalWriter` stays available for tests | Medium | Activates code that already exists in the architecture but has never been exercised end-to-end; prevents the locked-block invariant from being violated on restart |
| H7 | **Fleet TOML configuration** — a single `cluster.toml` file that declares the entire fleet: node count, each node's peer ID, listen address (transport + external interface), sled DB path, log level, and the validator set. `etheram-desktop` reads this file at startup and wires all nodes from it — no recompilation needed to change topology. Individual node sections can override fleet-level defaults | Low-Medium | Makes the cluster genuinely operable without recompiling; validates that `EtheramNodeBuilder` can be driven entirely from external config |
| H8 | **Network partition simulation** — a `PartitionableTransport` decorator wraps any `TransportIncoming`/`TransportOutgoing` implementation and intercepts messages based on a runtime-configurable partition table (`BTreeSet<(PeerId, PeerId)>` of blocked links). The partition table is updated via a control gRPC endpoint on each node process. `etheram-desktop` sends partition/heal commands to the affected processes and reflects the state in the dashboard (`partition <nodeA> <nodeB>` CLI command + hotkey) | Medium | Makes network partitioning a first-class, scriptable test stimulus — enables live demonstration of BFT tolerance under split-brain conditions and liveness recovery after healing |

### Crate layout

```
etheram-node-process/   # binary crate — runs exactly one node, driven by cluster.toml + node id arg
  src/
    main.rs             # parses args, reads cluster.toml, wires and runs EtheramNode step loop
    cluster_config.rs   # TOML deserialization (serde) for fleet + per-node config
    grpc_transport/     # tonic-based TransportIncoming + TransportOutgoing (H1)
    grpc_external/      # tonic-based ExternalInterfaceIncoming + Outgoing (H5)
    sled_storage/       # sled-based StorageAdapter (H2)
    wal_writer/         # sled/file-backed WalWriter (H6)
    partitionable_transport/  # partition-table transport decorator (H8)
    process_observer/   # structured-log Observer writing to stdout for launcher consumption

etheram-desktop/        # binary crate — launcher + dashboard, spawns child node processes
  src/
    main.rs             # reads cluster.toml, spawns N etheram-node-process children + UI
    launcher.rs         # child process lifecycle (spawn, kill, restart, health-check)
    ui/                 # ratatui dashboard, fed by child stdout and gRPC health queries
```

`etheram-desktop` depends on `etheram-node` and `core` only — same dependency
rule as `etheram-embassy`. It is `std`-only and explicitly not `no_std`.

### `cluster.toml` structure (H7)

```toml
[fleet]
validator_set = [1, 2, 3, 4, 5]
log_level = "info"

[[node]]
id = 1
transport_addr = "127.0.0.1:7001"
client_addr    = "127.0.0.1:8001"
db_path        = "./data/node1"

[[node]]
id = 2
transport_addr = "127.0.0.1:7002"
client_addr    = "127.0.0.1:8002"
db_path        = "./data/node2"
# ... one [[node]] section per node
```

### Suggested scenario

1. Author a `cluster.toml` for 5 nodes and start the desktop app — all nodes elect a leader and begin committing empty blocks.
2. Submit several transactions via gRPC (H5); watch them appear as pending and then committed in the dashboard.
3. Use the partition command (`partition 1 2`) to split the cluster; observe the dashboard detect the degraded state.
4. Heal the partition (`heal 1 2`); verify the cluster resumes committing.
5. Kill one node process (SIGTERM or dashboard `kill` command); verify the remaining 4 continue committing (BFT tolerance: `f=1` for `n=5`).
6. Restart the killed node process with the same `cluster.toml` entry; verify it recovers from WAL and catches up.

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
