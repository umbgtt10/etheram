# Raft Consensus — Implementation Plan

**Goal:** Implement Raft as a second consensus protocol to prove the 3-6 architectural model generalizes across consensus families
**Approach:** Option B — independent crate family (`raft-node`, `raft-variants`, `raft-validation`, `raft-embassy`), each depending only on `core/`
**Reference:** MetalRaft (`C:\Projects\metal_raft`) — full Raft implementation with 12 generic type parameters, mutable context, inline I/O
**Related:** [IBFT Roadmap](IBFT-ROADMAP.md) — Ethereum-specific IBFT consensus

---

## Architectural Decision: Why a Separate Crate Family

The EtheRAM crate family (`etheram/`, `etheram-variants/`, `etheram-validation/`, `etheram-embassy/`) is Ethereum-specific at every layer: `Action<M>` carries `UpdateAccount`, `StoreBlock`, `ExecuteBlock`; `Context` carries `accounts`, `state_root`, `pending_txs`; `TimerEvent` carries `ProposeBlock`, `TimeoutRound`. These types are not generic — they are protocol-specific by design (Architecture Decision 8: Protocol-Specific Types).

Raft requires fundamentally different types: `RaftAction<P>` with `SetTerm`, `AppendLog`, `AdvanceCommitIndex`; `RaftContext<P>` with `current_term`, `voted_for`, `log`, `commit_index`; `RaftTimerEvent` with `ElectionTimeout`, `Heartbeat`.

**Option A** (generalize `etheram/`) would risk the 557 existing tests and blur the Ethereum-specific semantics that make the IBFT implementation meaningful.

**Option B** (independent crate family) is chosen because:
1. **Independent emergence** — the 3-6 pattern independently re-emerging for Raft is stronger architectural evidence than parameterization
2. **Zero risk** — no changes to `core/`, `etheram/`, `etheram-variants/`, `etheram-validation/`, or `etheram-embassy/`
3. **Protocol-specific types** — each protocol defines exactly what it needs, no compromises
4. **Dedicated Embassy crate** — avoids feature flag explosion (6 per crate vs 12+ shared) and keeps IBFT/Raft scenarios isolated

### Target Workspace Structure

```
core/                  Shared: PeerId, ConsensusProtocol trait (unchanged)
etheram/               Ethereum node — 3-6 model with IBFT (unchanged)
etheram-variants/      IBFT concrete implementations (unchanged)
etheram-validation/    IBFT cluster tests (unchanged)
etheram-embassy/       IBFT on Embassy — 2 configs (unchanged)
raft-node/             Raft node — 3-6 model with Raft (NEW)
raft-variants/         Raft concrete implementations (NEW)
raft-validation/       Raft cluster tests (NEW)
raft-embassy/          Raft on Embassy — 2 configs (NEW)
```

### Crate Dependency Graph

```
               ┌── etheram ←── etheram-variants ←── etheram-validation
               │                               ←── etheram-embassy
core ──────────┤
               │
               └── raft-node ←── raft-variants ←── raft-validation
                                               ←── raft-embassy
```

Both protocol families share only `core/` (PeerId, `ConsensusProtocol` trait, dimension I/O traits). No cross-dependencies between the two families.

---

## IBFT vs Raft: Maximum Architectural Difference

The choice of Raft is deliberate — it is the most different consensus protocol from IBFT across every axis:

| Axis | IBFT | Raft |
|---|---|---|
| Fault model | Byzantine (⅓ tolerance) | Crash-only (½ tolerance) |
| Quorum | `⌊2n/3⌋ + 1` | `⌊n/2⌋ + 1` |
| Leader election | Round-robin deterministic | Randomized timeout + vote |
| Commit flow | 3-phase (PrePrepare→Prepare→Commit) | 2-phase (AppendEntries + majority ack) |
| Log model | Block-at-a-time (single pending block) | Append-only log (unbounded entries) |
| State machine | Ethereum accounts, EVM execution | Generic key-value |
| Recovery model | WAL + prepared certificate | Persistent log + snapshot |

If the 3-6 decomposition accommodates both, no consensus protocol is outside its scope.

---

## Scope: Essential Raft Features

### In Scope (Sprints 0-6)

| Feature | Description | MetalRaft Reference |
|---|---|---|
| Leader election | Candidate → RequestVote → majority → Leader | `election.rs` |
| Pre-vote | PreVote phase prevents disruptive re-elections | `election.rs` |
| Log replication | AppendEntries from leader to followers | `replication.rs` |
| Heartbeat | Periodic empty AppendEntries to maintain leadership | `replication.rs` |
| Commit advancement | `commit_index` advances when majority acks a log entry | `replication.rs` |
| State machine apply | Apply committed entries to key-value state machine | `state_machine.rs` |
| Step-down | Leader steps down on higher term or election timeout | `common.rs` |
| Basic snapshot | Single-shot snapshot for state transfer | `snapshot.rs` |
| Client command | Submit key-value commands, receive applied results | `RaftMsg::ClientCommand` |
| Observer | Per-action callbacks matching IBFT observer pattern | `observer.rs` |

### Deferred (Post-MVP)

| Feature | Reason to Defer |
|---|---|
| Chunked InstallSnapshot | Optimization — single-shot is sufficient for correctness |
| Configuration changes (AddNode/RemoveNode) | Complex; not needed to prove 3-6 decomposition |
| Leader lease / read optimization | Performance optimization, not correctness |
| Read-only queries via leader lease | Extension of leader lease |
| Batched replication | Performance optimization |

---

## MetalRaft → 3-6 Model Translation

The core architectural transformation: MetalRaft's mutable, imperative `MessageHandler` with inline I/O becomes a pure, declarative `RaftProtocol` returning `RaftAction` variants.

### Side-Effect Translation Table

| MetalRaft (imperative) | 3-6 Model (declarative action) |
|---|---|
| `self.storage.set_current_term(term)` | `RaftAction::SetTerm(term)` |
| `self.storage.set_voted_for(peer)` | `RaftAction::SetVotedFor(peer)` |
| `self.storage.append_entries(entries)` | `RaftAction::AppendEntries(entries)` |
| `self.storage.truncate_from(index)` | `RaftAction::TruncateLogFrom(index)` |
| `self.transport.send(peer, msg)` | `RaftAction::SendMessage(peer, msg)` |
| `self.transport.broadcast(msg)` | `RaftAction::BroadcastMessage(msg)` |
| `self.timer.reset(ElectionTimeout)` | `RaftAction::ScheduleTimeout(ElectionTimeout)` |
| `self.timer.reset(Heartbeat)` | `RaftAction::ScheduleTimeout(Heartbeat)` |
| `self.state_machine.apply(entry)` | `RaftAction::ApplyToStateMachine(entry)` |
| `self.observer.election_won(term)` | `RaftAction::Log(...)` |
| `context.commit_index = new_index` | `RaftAction::AdvanceCommitIndex(new_index)` |
| `context.role = Leader` | `RaftAction::TransitionRole(Leader)` |
| `self.storage.save_snapshot(...)` | `RaftAction::SaveSnapshot(snapshot)` |

### Type Mapping

| MetalRaft Type | 3-6 Raft Type | Notes |
|---|---|---|
| `RaftMsg<P, L, C>` (8 variants) | `RaftMessage<P>` | Single generic param; log/command embedded |
| `Event<P, L, C>` | `Message<RaftMessage<P>>` | Unified through `MessageSource` |
| `TimerKind` (Election, Heartbeat) | `RaftTimerEvent` | Protocol-specific timer events |
| `Storage` (trait, ~20 methods) | `RaftStorageQuery` + `RaftStorageMutation` | Separated into query/mutate |
| `StateMachine` (trait) | `RaftStateMachine` (trait) | Key-value apply + get + snapshot |
| `MessageHandlerContext` (mutable) | `RaftContext<P>` (immutable) | Context built fresh each step |
| `Transport` (trait, sends inline) | `TransportOutgoingAdapter` | Output actions dispatched by executor |

---

## Sprint Plan

### Sprint 0: Crate Skeleton (0.5 days)

Create `raft-node/` with Raft-specific types:

| Type | File | Purpose |
|---|---|---|
| `RaftMessage<P>` | `brain/protocol/message.rs` | RequestVote, RequestVoteResponse, PreVoteRequest, PreVoteResponse, AppendEntries, AppendEntriesResponse, InstallSnapshot, InstallSnapshotResponse |
| `RaftAction<P>` | `brain/protocol/action.rs` | SetTerm, SetVotedFor, AppendEntries, TruncateLogFrom, SendMessage, BroadcastMessage, ScheduleTimeout, ApplyToStateMachine, AdvanceCommitIndex, TransitionRole, SaveSnapshot, SendClientResponse, Log |
| `RaftContext<P>` | `context/context_dto.rs` | current_term, voted_for, log, commit_index, last_applied, role, leader_id, peers, match_index, next_index |
| `RaftTimerEvent` | `incoming/timer/timer_event.rs` | ElectionTimeout, Heartbeat |
| `RaftClientRequest<C>` | `incoming/external_interface/client_request.rs` | Command(C), Query(key) |
| `RaftClientResponse` | `executor/outgoing/external_interface/client_response.rs` | Applied(result), NotLeader(leader_id), Timeout |
| `RaftStorageQuery` | `state/storage_query.rs` | CurrentTerm, VotedFor, LogEntries, LogLength, LastLogTerm, Snapshot |
| `RaftStorageMutation` | `state/storage_mutation.rs` | SetTerm, SetVotedFor, AppendEntries, TruncateFrom, SaveSnapshot |
| `RaftCacheQuery` | `state/cache_query.rs` | CommitIndex, LastApplied, Role, MatchIndex, NextIndex |
| `RaftCacheUpdate` | `state/cache_update.rs` | SetCommitIndex, SetLastApplied, SetRole, UpdateMatchIndex, UpdateNextIndex |
| `NodeRole` | `common_types/node_role.rs` | Follower, Candidate, Leader |
| `LogEntry<P>` | `common_types/log_entry.rs` | term, index, payload |

`raft-node/` mirrors `etheram/` structurally but with Raft-specific types. `#![no_std]` from day one.

### Sprint 1: RaftNode Step Loop (0.5 days)

Build `RaftNode<P>` with the same step() primitive:

```rust
pub struct RaftNode<P: Clone + 'static> {
    peer_id: PeerId,
    incoming: RaftIncomingSources<P>,
    state: RaftState,
    executor: RaftExecutor<P>,
    context_builder: Box<dyn RaftContextBuilder<P>>,
    brain: BoxedRaftProtocol<P>,
    partitioner: Box<dyn RaftPartitioner<P>>,
    state_machine: Box<dyn RaftStateMachine>,
    observer: Box<dyn RaftObserver>,
}
```

The `RaftPartitioner` produces a 2-way partition: `(mutations, outputs)`. No `executions` — Raft has no block execution engine. State machine apply is an output action.

### Sprint 2: RaftProtocol (3 days)

Implement `RaftProtocol<P>` as a pure `ConsensusProtocol` implementation. This is the core translation sprint — every MetalRaft `MessageHandler` method becomes a declarative action-returning function.

Sub-modules:
- `election.rs` — handle_election_timeout, handle_request_vote, handle_request_vote_response, handle_pre_vote_request, handle_pre_vote_response
- `replication.rs` — handle_append_entries, handle_append_entries_response, handle_heartbeat_timeout, replicate_to_peer
- `snapshot.rs` — handle_install_snapshot, handle_install_snapshot_response, trigger_snapshot
- `common.rs` — step_down_if_higher_term, advance_commit_index, apply_committed_entries

Each handler reads from `RaftContext<P>` (immutable) and returns `ActionCollection<RaftAction<P>>`. No I/O, no mutation.

### Sprint 3: Concrete Implementations (1 day)

Create `raft-variants/` with:

| Component | Type | Description |
|---|---|---|
| Storage | `InMemoryRaftStorage` | `BTreeMap`-backed term, voted_for, log |
| Cache | `InMemoryRaftCache` | commit_index, last_applied, role, match/next_index |
| Transport | `InMemoryRaftTransport` | Shared state for cluster testing |
| Transport | `NoOpRaftTransport` | Silent (unit testing) |
| Timer | `InMemoryRaftTimer` | Deterministic test driving |
| External Interface | `InMemoryRaftExternalInterface` | Client command submission |
| Context Builder | `EagerRaftContextBuilder` | Reads all state eagerly |
| Partitioner | `TypeBasedRaftPartitioner` | 2-way partition by action variant |
| State Machine | `InMemoryRaftStateMachine` | Key-value `BTreeMap<String, String>` |
| Observer | `NoOpRaftObserver` | Silent |
| Builder | `RaftNodeBuilder` | Builder pattern for node construction |

### Sprint 4: Stage 1 Tests (~111 tests, 2 days)

Protocol-level tests in `raft-variants/tests/`:

| Area | Tests | Coverage |
|---|---|---|
| Election | ~20 | Timeout → candidate → vote → leader; split vote; pre-vote; higher term step-down |
| Replication | ~25 | AppendEntries; log matching; conflict resolution; commit advancement |
| Snapshot | ~10 | Trigger; install; state restore |
| Role transitions | ~10 | Follower→Candidate→Leader; Leader→Follower; term bumps |
| Message handling | ~15 | Reject stale messages; handle all 8 message types |
| Client commands | ~10 | Command submission; not-leader redirect; applied response |
| Timer | ~8 | Election timeout; heartbeat scheduling; reset on message |
| Storage/Cache | ~13 | InMemoryRaftStorage queries/mutations; InMemoryRaftCache operations |

### Sprint 5: Stage 2 Tests (~56 tests, 2 days)

Cluster-level tests in `raft-validation/`:

| Area | Tests | Coverage |
|---|---|---|
| Election | ~12 | 5-node election; network partition; split vote recovery; pre-vote prevents disruption |
| Replication | ~15 | Leader replicates to all; follower catches up; log consistency after partition heal |
| Fault tolerance | ~10 | Minority crash → progress; majority crash → stall; leader crash → re-election |
| Snapshots | ~6 | Slow follower gets snapshot; snapshot + append hybrid |
| State machine | ~8 | Client command → applied across cluster; linearizable read after commit |
| Client interface | ~5 | Submit to leader; redirect on follower; timeout on partition |

### Sprint 6: Stage 3 — Embassy (2 days)

Create `raft-embassy/` with two configurations:

| Configuration | Transport | Storage | External Interface | Script |
|---|---|---|---|---|
| **all-in-memory** | `channel-transport` | `in-memory-storage` | `channel-external-interface` | `run_raft_channel_in_memory.ps1` |
| **real** | `udp-transport` | `semihosting-storage` | `udp-external-interface` | `run_raft_udp_semihosting.ps1` |

5-act Raft scenario:
- Act 0: Election — 5 nodes boot, one wins election, becomes leader
- Act 1: Replication — client submits key-value command, leader replicates, majority acks, command applied
- Act 2: Read-after-write — query the key, verify value matches
- Act 3: Leader crash — kill leader task, new election, new leader emerges
- Act 4: Continued replication — new command submitted to new leader, replicated and applied

### Sprint 7: Documentation (0.5 days)

- `raft-node/README.md` — crate documentation
- `raft-variants/README.md` — implementations documentation
- `raft-validation/README.md` — cluster test documentation
- `raft-embassy/README.md` — Embassy port documentation
- Update root `README.md` — add Raft crates to workspace structure, update test count, add Raft to "Achieved"
- Update `docs/ARCHITECTURE.md` — reference both protocol families as evidence
- Update ADRs — update evidence sections with Raft data
- Update `copilot-instructions.md` — add Raft crate rules and constraints
- Update `scripts/test.ps1` — add Raft crates to CI gate

---

## Estimated Effort

| Sprint | Days | Deliverable |
|---|---|---|
| 0 — Crate skeleton | 0.5 | `raft-node/` types |
| 1 — RaftNode step loop | 0.5 | Working `step()` with NoOp protocol |
| 2 — RaftProtocol | 3.0 | Pure Raft consensus (election + replication + snapshot) |
| 3 — Concrete implementations | 1.0 | InMemory* + builders |
| 4 — Stage 1 tests | 2.0 | ~111 protocol-level tests |
| 5 — Stage 2 tests | 2.0 | ~56 cluster-level tests |
| 6 — Stage 3 Embassy | 2.0 | 2 configurations, 5-act QEMU scenario |
| 7 — Documentation | 0.5 | READMEs, ADRs, ARCHITECTURE updates |
| **Total** | **~11.5** | |

---

## Success Criteria

1. **Same decomposition** — `RaftNode<P>` has the same 6-dimension structure as `EtheramNode<M>`
2. **Same step() primitive** — identical poll → context → handle → partition → apply → execute flow
3. **Pure protocol** — `RaftProtocol<P>.handle_message()` takes immutable context, returns declarative actions
4. **Same trait** — `RaftProtocol<P>` implements `ConsensusProtocol` from `core/`
5. **Same 3-stage validation** — protocol tests, cluster tests, and embedded QEMU end-to-end
6. **Same Embassy pattern** — `select4` reactor with `while node.step() {}`
7. **Zero changes to existing crates** — `core/`, `etheram/`, `etheram-variants/`, `etheram-validation/`, `etheram-embassy/` remain untouched
8. **All existing tests pass** — 557 tests remain green throughout

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Raft log model doesn't fit `StorageQuery`/`StorageMutation` pattern | Low | Raft-specific query/mutation types (independent from Ethereum types) |
| `ConsensusProtocol` trait too Ethereum-specific | Low | Already verified — trait is generic over message and action types |
| Embassy feature flag conflicts | Low | Separate crate (`raft-embassy/`) eliminates cross-contamination |
| MetalRaft mutable context hard to make pure | Low | Same transformation done for IBFT; mechanical mapping |
| Snapshot state transfer too complex for MVP | Medium | Defer chunked InstallSnapshot; single-shot snapshot is sufficient |
