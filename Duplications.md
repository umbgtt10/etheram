# Duplications — Current Status

Cross-family code duplication identified during the March 2026 code review. Most protocol-agnostic items have been consolidated into shared crates (`core/`, `embassy-core/`). Remaining items below are intentionally protocol-specific or pending further consolidation.

---

## 1. `SharedState<T>` Trait

- `etheram-node/src/implementations/shared_state.rs`
- `raft-node/src/implementations/shared_state.rs`

**Status:** Consolidated in `core/src/node_common/shared_state.rs`.
**Fix:** Completed.

---

## 2. `StdSharedState<T>` Implementation

- `core/src/node_common/shared_state.rs` (`StdSharedState<T>`)
- `core/tests/node_common/std_shared_state_tests.rs`

**Status:** Consolidated in `core` and redundant validation copies removed.
**Fix:** Completed.

---

## 3. `BuildError` Enum

- `etheram-node/src/builders/error.rs`
- `raft-node/src/builders/error.rs`

**Status:** Consolidated in `core/src/node_common/build_error.rs`.
**Fix:** Completed.

---

## 4. `ActionCollection<T>`

- `etheram-node/src/collections/action_collection.rs`
- `raft-node/src/collections/action_collection.rs`

**Status:** Near-identical `Vec<T>` wrapper implementing `Collection`. Raft version missing: inherent `new()`, `Default`, `IntoIterator`.
**Fix:** Move to `core/` as the canonical `Collection` implementation. Add missing impls to raft first.

---

## 5. `EventLevel` / `RaftEventLevel` Enum

- `etheram-node/src/observer.rs` (`EventLevel`)
- `raft-node/src/observer.rs` (`RaftEventLevel`)

**Status:** Identical variants (`None, Essential, Info, Debug, Trace`). Protocol-agnostic.
**Fix:** Move to `core/src/event_level.rs`.

---

## 6. Generic I/O Adapter Traits (6 of 8)

- `TimerInputAdapter`, `TimerOutputAdapter`, `ExternalInterfaceIncomingAdapter`, `ExternalInterfaceOutgoingAdapter`, `TransportIncomingAdapter`, `TransportOutgoingAdapter`

**Status:** Consolidated in `core/src/node_common/` for protocol-agnostic adapters. `StorageAdapter` and `CacheAdapter` remain protocol-specific by design.
**Fix:** Completed.
