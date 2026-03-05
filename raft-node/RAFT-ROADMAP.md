# Raft Consensus — Feature Status

**Scope:** Raft consensus protocol in the independent `raft-*` crate family
**Implementation:** `raft-node/src/implementations/raft/`
**Validation model:**
- Stage 1: protocol tests in `raft-node/tests/implementations/raft_protocol/`
- Stage 2: cluster tests in `raft-validation/tests/`
- Stage 3: embedded end-to-end in `raft-embassy/`

**Related:** [IBFT Roadmap](IBFT-ROADMAP.md) — Ethereum-specific IBFT consensus

---

## Supported Features

### Core Consensus

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| Leader election | Candidate → `RequestVote` → majority → Leader | ✅ | ✅ | ✅ |
| Pre-vote | `PreVoteRequest` / `PreVoteResponse` phase prevents disruptive re-elections | ✅ | ✅ | ✅ |
| Log replication | Leader sends `AppendEntries` to followers; followers respond with `match_index` | ✅ | ✅ | ✅ |
| Heartbeat | Periodic empty `AppendEntries` to maintain leadership (`HEARTBEAT_INTERVAL_MS = 100`) | ✅ | ✅ | ✅ |
| Commit advancement | `commit_index` advances when majority of peers ack a log entry in the leader's current term | ✅ | ✅ | ✅ |
| Quorum computation | `⌊n/2⌋ + 1` via `common::quorum_size()` | ✅ | ✅ | ✅ |
| State machine apply | Committed entries applied to key-value `RaftStateMachine` via `ApplyToStateMachine` actions | ✅ | ✅ | ✅ |
| Single-node cluster | Leader election and operation with zero peers | ✅ | — | — |

### Safety and Robustness

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| Step-down on higher term | Any message with a higher term triggers `SetTerm` + `TransitionRole(Follower)` + `SetVotedFor(None)` | ✅ | ✅ | ✅ |
| Log consistency check | `AppendEntries` rejected when `prev_log_index` / `prev_log_term` don't match local log | ✅ | ✅ | — |
| Log conflict resolution | Conflicting entries detected and truncated; new entries appended from conflict point | ✅ | ✅ | — |
| Election timeout reset | `ScheduleTimeout(ElectionTimeout)` emitted on valid `AppendEntries`, `RequestVote` grant, and `InstallSnapshot` | ✅ | ✅ | ✅ |
| Candidate step-down | Candidate transitions to Follower on receiving `AppendEntries` with current or higher term | ✅ | ✅ | — |
| Vote persistence | `SetVotedFor` action emitted before `RequestVoteResponse` — vote is persisted before reply | ✅ | ✅ | ✅ |
| Log up-to-date check | `RequestVote` and `PreVoteRequest` compare last log term + index for candidate eligibility | ✅ | ✅ | ✅ |
| Stale message rejection | Messages with terms lower than `current_term` are rejected with current term in response | ✅ | ✅ | — |

### Snapshot and Recovery

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| Snapshot install | `InstallSnapshot` from leader installs snapshot, truncates log, restores state machine | ✅ | ✅ | — |
| Snapshot response | Follower responds with success/failure; leader updates `match_index` / `next_index` on success | ✅ | ✅ | — |
| Stale snapshot skip | Snapshot with `snapshot_index <= commit_index` acknowledged but not installed | ✅ | ✅ | — |
| Snapshot-aware log helpers | `last_log_index`, `last_log_term`, `prev_log_term_at` fall back to snapshot metadata when log is empty | ✅ | ✅ | — |

### Client Interface

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| Command submission | `RaftClientRequest::Command` appends entry to leader's log and triggers replication | ✅ | ✅ | ✅ |
| Query routing | `RaftClientRequest::Query` dispatches `QueryStateMachine` on leader, `NotLeader` redirect otherwise | ✅ | ✅ | ✅ |
| Not-leader redirect | Non-leader nodes respond with `RaftClientResponse::NotLeader(leader_id)` | ✅ | ✅ | ✅ |
| Applied response | `SendClientResponse(Applied)` emitted when a client's entry is committed and applied | ✅ | ✅ | ✅ |
| Client tracking | `pending_client_entries: BTreeMap<u64, ClientId>` maps log index → client for response routing | ✅ | ✅ | ✅ |

### Node Infrastructure

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| 6-dimension `RaftNode<P>` | Peer, IncomingSources, State, Executor, ContextBuilder, Protocol, Partitioner, StateMachine, Observer | ✅ | ✅ | ✅ |
| `RaftNodeBuilder<P>` | Builder pattern for node construction with variant enums | ✅ | ✅ | ✅ |
| 2-way partitioner | `TypeBasedRaftPartitioner` splits actions into `(mutations, outputs)` — no execution tier | ✅ | ✅ | ✅ |
| `RaftObserver` trait | `RaftActionKind` projection with `action_emitted`, `mutation_applied`, `output_executed` callbacks | ✅ | ✅ | ✅ |
| `EagerRaftContextBuilder` | Reads all storage + cache state into immutable `RaftContext<P>` each step | ✅ | ✅ | ✅ |
| `InMemoryRaftStorage<P>` | `BTreeMap`-backed term, voted_for, log, snapshot | ✅ | ✅ | ✅ |
| `InMemoryRaftCache` | commit_index, last_applied, role, leader_id, match_index, next_index | ✅ | ✅ | ✅ |
| `InMemoryRaftTransport<P,S>` | Shared-state transport for cluster testing | — | ✅ | ✅ |
| `InMemoryRaftTimer<S>` | Deterministic timer for test driving (schedule is no-op; events pushed externally) | ✅ | ✅ | ✅ |
| `InMemoryRaftExternalInterface<S>` | Channel-like client request/response for test driving | ✅ | ✅ | ✅ |
| `InMemoryRaftStateMachine` | Key-value `BTreeMap` with `apply`, `get`, `snapshot`, `restore` | ✅ | ✅ | ✅ |
| `NoOpRaftTransport<P>` | Silent transport for isolated protocol testing | ✅ | — | — |
| `NoOpRaftObserver` | Silent observer | ✅ | ✅ | — |

### Embassy (Stage 3)

| Feature | Description | In-Memory | Real |
|---|---|---|---|
| Channel transport | Embassy `Channel`-based transport hub with outbox bridge | ✅ | — |
| UDP transport | Postcard-serialized `RaftMessage` over UDP | — | ✅ |
| In-memory storage | `InMemoryRaftStorage` in Embassy context | ✅ | — |
| Semihosting storage | Mutation-counting storage with ARM-gated semihosting logging | — | ✅ |
| Channel external interface | Embassy channel-based client request/response | ✅ | — |
| UDP external interface | UDP-backed client interface | — | ✅ |
| 5-act QEMU scenario | Election → Replication → Read-after-write → Re-election → Continued replication | ✅ | ✅ |
| Cancellation token | Graceful shutdown via shared `CancellationToken` | ✅ | ✅ |

---

## Planned Features

| Feature | Description | Priority | Complexity |
|---|---|---|---|
| Configuration changes (joint consensus) | `AddServer` / `RemoveServer` with two-phase joint-consensus per Raft extended paper | Medium | High |
| Read leases | `ReadIndex` and/or lease-based reads avoiding full log replication for queries | Medium | Medium |
| Chunked InstallSnapshot | Stream large snapshots in chunks instead of single-shot transfer | Low | Medium |
| Leader lease | Leader maintains lease to serve reads without quorum confirmation | Low | Medium |
| Batched replication | Bundle multiple log entries per `AppendEntries` round for throughput | Low | Low |
| Log compaction trigger | Automatic snapshot creation when log exceeds configurable size | Medium | Low |
| Real snapshot storage | `SnapshotStorage` dimension serializing snapshots to semihosting files | Medium | Medium |
| Embassy fault injection | Kill a node mid-replication in QEMU, verify 3-of-5 survival | Medium | Low |

---

## Test Coverage Summary

| Area | Stage 1 (protocol) | Stage 2 (cluster) | Stage 3 (QEMU) |
|---|---|---|---|
| Election / pre-vote | ✅ | ✅ | ✅ (Act 0) |
| Log replication / AppendEntries | ✅ | ✅ | ✅ (Act 1) |
| Commit advancement | ✅ | ✅ | ✅ (Acts 1, 4) |
| Heartbeat | ✅ | ✅ | — |
| Role transitions | ✅ | ✅ | ✅ (Act 3) |
| Step-down on higher term | ✅ | ✅ | — |
| Log conflict resolution | ✅ | ✅ | — |
| Snapshot install / restore | ✅ | ✅ | — |
| Client command / response | ✅ | ✅ | ✅ (Acts 1, 4) |
| Client query (read-after-write) | ✅ | ✅ | ✅ (Act 2) |
| Not-leader redirect | ✅ | ✅ | ✅ |
| Fault tolerance (minority crash) | — | ✅ | — |
| State machine apply | ✅ | ✅ | ✅ |
| InMemoryRaftStorage | ✅ | — | — |
| InMemoryRaftCache | ✅ | — | — |
| Snapshot state transfer too complex for MVP | Medium | Defer chunked InstallSnapshot; single-shot snapshot is sufficient |
