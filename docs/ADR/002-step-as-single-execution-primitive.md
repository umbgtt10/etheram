# ADR-002: step() as Single Execution Primitive

## Status

**Accepted** — Validated by Etheram and Raft implementations (January–March 2026)

## Context

Distributed systems nodes can execute in many ways:
- **Sequential loops** (blocking, simple)
- **Async tasks** (tokio, async-std)
- **Embedded async** (Embassy, RTIC)
- **Thread-based** (OS threads, work-stealing)
- **Process-based** (actor model, message passing)

Each execution model has different APIs, scheduling semantics, and runtime requirements. If we couple node logic to a specific model, we cannot:
- Test deterministically (async is non-deterministic)
- Deploy to embedded systems (no tokio, limited threads)
- Compare execution strategies (locked into one)

We need a **minimal, universal primitive** that works in all environments while enabling deterministic testing.

## Decision

We define **`step()`** as the single required execution method:

```rust
pub fn step(&mut self) -> bool;
```

**Semantics:**
1. Polls all event sources (transport, external_interface, timer) in deterministic order via `IncomingSources::poll()`
2. Processes exactly one event (if available)
3. Builds context via `ContextBuilder` (immutable snapshot of current state)
4. Passes event to protocol (`brain.handle_message()`) which returns declarative actions
5. Partitions actions into mutations, outputs, and executions via `Partitioner`
6. Applies mutations to state, dispatches outputs to executor, runs execution engine for block executions
7. Notifies `Observer` at each phase (context built, action emitted, mutation applied, output executed)
8. Returns immediately (no blocking, no async)
9. Returns `true` if work was done, `false` if idle

**All execution patterns build on step():**

### Blocking Loop
```rust
loop {
    node.step();
}
```

### Run Until Idle
```rust
while node.step() {
    // Continue until no work
}
```

### Deterministic Test Orchestration
```rust
cluster.fire_timer(0, TimerEvent::ProposeBlock);
cluster.step_all();  // calls step() on each node until idle
cluster.step(1);     // step a single node
```

### Embassy Embedded (Validated)
```rust
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
            Either4::Third(timer_event) => {
                timer_state.push_event(peer_id, timer_event);
                while node.step() {}
            }
            Either4::Fourth(()) => {
                while node.step() {}
            }
        }
    }
}
```

**Key Insight:** Execution model becomes an **implementation detail**, not a core abstraction. The three validated environments (sequential test loop, cluster orchestrator, Embassy async task) all use the same `EtheramNode::step()` method with zero changes to the node logic.

## Consequences

### Positive

1. **Environment Agnostic** — Same primitive works everywhere: std tests, cluster validation, no_std ARM Cortex-M4 via QEMU
2. **Deterministic Testing** — Call `step()` explicitly in tests, full control over event ordering and interleaving across 748 tests
3. **Debuggable** — Step through one event at a time, inspect state between steps; Observer trait provides per-step visibility
4. **Composable** — Build any execution pattern from `step()`: blocking loops, run-until-idle, async event-driven (Embassy `select4`), cluster orchestration
5. **Simple Contract** — One method, clear semantics, easy to implement
6. **No Runtime Lock-in** — Not tied to tokio, Embassy, OS threads, or any framework; proven across std and no_std
7. **Reactive Integration** — Embassy wrapper uses `select4` to await on transport/timer/EI channels, then drives `step()` reactively — no busy-waiting, no polling overhead

### Negative

1. **Polling Overhead** — If called without events, returns immediately with `false` (mitigated by event-driven wrappers like Embassy `select4`)
2. **No Built-in Backpressure** — Caller must implement rate limiting (flexibility vs convenience)
3. **Manual Orchestration** — No automatic scheduling (intentional — caller controls execution order for determinism)

### Validation Evidence

Etheram validates `step()` as the universal execution primitive across three environments:

**Environment 1 — Sequential testing (std):**
- 748 tests execute deterministically via explicit `step()` calls
- `IbftCluster::step_all()` calls `step()` on each node in sequence until all are idle
- `IbftCluster::step(node_index)` enables per-node stepping for fine-grained interleaving tests
- `IbftTestNode` wraps a single `EtheramNode` and drives `step()` for isolated protocol testing
- Run-until-idle (`while node.step() {}`) is the standard test execution pattern

**Environment 2 — Cluster orchestration (std):**
- `IbftCluster` with 4–7 validator nodes, all using shared in-memory transport
- Byzantine fault injection, message interception, round progression — all driven by `step()`
- Event injection (`fire_timer`, `submit_request`, `push_transport_message`) followed by `step_all()` — no async, no threads

**Environment 3 — Embedded async (no_std, ARM Cortex-M4, QEMU):**
- `#[embassy_executor::task(pool_size = 5)]` spawns 5 concurrent IBFT node tasks
- Each task wraps `step()` inside a `select4` reactor: awaits transport inbound, timer events, EI notifications, or cancellation
- On event arrival, injects event into the synchronous dimension, then calls `while node.step() {}` to drain all resulting work
- Two independently maintained configurations (all-in-memory and UDP+semihosting) both use the same `step()` contract
- 12-act scenario (transfers, view changes, overdrafts, gas limits, validator set updates, WAL persistence, Ed25519 signatures, TinyEVM contract execution) validates real protocol behaviour end-to-end
- Graceful shutdown via `CancellationToken` — `select4` returns `Either4::First(())` to break the loop

**The step() primitive is unchanged across all three environments.** Only the wrapper — how events are produced and when `step()` is called — varies.

Raft independently validates the same `step()` primitive with the mirrored execution shape:

**Environment 1 — Sequential/protocol testing (std):**
- `RaftProtocol<P>` behaviour validated via deterministic event-to-action tests

**Environment 2 — Cluster orchestration (std):**
- `RaftCluster` drives multi-node execution with explicit `step()`, `drain()`, and timer/message injection controls

**Environment 3 — Embedded async (no_std, ARM Cortex-M4, QEMU):**
- Embassy task wrappers use async event waiting, then drain synchronous work with `while node.step() {}`
- Both required configurations (all-in-memory and UDP+semihosting) run end-to-end
- 5-act Raft scenario validates election, replication, read-after-write, re-election, and continued replication

This cross-family replication of execution semantics confirms that `step()` is a protocol-agnostic primitive, not an IBFT-specific design choice.

## Related

- [ADR-001: Six-Dimension Node Decomposition](001-six-dimension-node-decomposition.md)
- [Architecture Documentation](../ARCHITECTURE.md)
