# ADR-002: step() as Single Execution Primitive

## Status

**Accepted** - Validated by TinyChain implementation (January 2026)

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
pub trait Node {
    /// Execute one unit of work
    /// Returns true if work was done, false if idle
    fn step(&mut self) -> bool;
}
```

**Semantics:**
1. Polls all event sources (transport, external_interface, timer) in deterministic order
2. Processes exactly one event (if available)
3. Applies resulting actions to dimensions
4. Returns immediately (no blocking, no async)
5. Returns `true` if work was done, `false` if idle

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

### Async Wrapper (future work)
```rust
async fn run_async(mut node: Node) {
    loop {
        if !node.step() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
```

### Embassy Wrapper (future work)
```rust
#[embassy_executor::task]
async fn run_embassy(mut node: Node) {
    loop {
        if !node.step() {
            Timer::after(Duration::from_millis(10)).await;
        }
    }
}
```

**Key Insight:** Execution model becomes an **implementation detail**, not a core abstraction.

## Consequences

### Positive

1. **Environment Agnostic** - Same primitive works everywhere (std, no-std, async, sync)
2. **Deterministic Testing** - Call `step()` explicitly in tests, full control over ordering
3. **Debuggable** - Step through one event at a time, inspect state between steps
4. **Composable** - Build any execution pattern (loops, async, reactors) from `step()`
5. **Simple Contract** - One method, clear semantics, easy to implement
6. **No Runtime Lock-in** - Not tied to tokio, Embassy, OS threads, or any framework

### Negative

1. **Polling Overhead** - Busy-waiting if not wrapped carefully (mitigated by wrappers)
2. **No Built-in Backpressure** - Caller must implement rate limiting (flexibility vs convenience)
3. **Manual Orchestration** - No automatic scheduling (intentional - caller controls)

### Validation Evidence

TinyChain demonstrates:
- ✅ 20 tests execute deterministically via explicit `step()` calls
- ✅ Sequential execution pattern works perfectly
- ✅ No async/await needed for core logic
- ✅ Test cluster controls exact execution order: `cluster.step(0)`, `cluster.step(1)`, etc.
- ✅ `run_until_idle()` utility built naturally from `step()`

Example test pattern:
```rust
cluster.inject_timer_event(1, TimerEvent::ProposeBlock);
cluster.step(1);  // Leader processes timer, creates block
cluster.step(0);  // Follower receives and accepts block
cluster.step(2);  // Another follower accepts
// Deterministic, repeatable, debuggable
```

### Future Work

- Async wrapper for tokio runtime
- Embassy wrapper for embedded async
- Demonstrate same Node running in all three environments

## Related

- [ADR-001: Six-Dimension Node Decomposition](001-six-dimension-node-decomposition.md)
- [Architecture Documentation](../ARCHITECTURE.md)
- TinyChain validation: `examples/tinychain/tests/validation.rs`
