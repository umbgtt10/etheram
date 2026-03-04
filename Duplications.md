# Duplications — To Be Fixed Later

Cross-family code duplication identified during the March 2026 code review. These items are deferred and should be consolidated into `core/` when convenient.

---

## 1. `SharedState<T>` Trait

- `etheram-variants/src/implementations/shared_state.rs`
- `raft-variants/src/implementations/shared_state.rs`

**Status:** Byte-for-byte identical. Protocol-agnostic.
**Fix:** Move to `core/src/shared_state.rs`. Both `*-variants` crates import from `core/`.

---

## 2. `StdSharedState<T>` Implementation

- `etheram-validation/src/std_shared_state.rs` (`StdSharedState<T>`)
- `raft-validation/src/std_raft_shared_state.rs` (`StdSharedState<T>` — after rename)

**Status:** Identical `Arc<Mutex<T>>` body.
**Fix:** Add as a `std`-feature-gated module in `core/`. Both validation crates import from `core/`.

---

## 3. `BuildError` Enum

- `etheram-variants/src/builders/error.rs`
- `raft-variants/src/builders/error.rs`

**Status:** Same single-variant enum `MissingComponent(&'static str)`. Minor derive divergence (etheram: `Debug, Clone`; raft: `Debug, Clone, Copy, PartialEq, Eq`).
**Fix:** Move to `core/src/builders/error.rs` with the union of derives plus `Display`.

---

## 4. `ActionCollection<T>`

- `etheram/src/collections/action_collection.rs`
- `raft-node/src/collections/action_collection.rs`

**Status:** Near-identical `Vec<T>` wrapper implementing `Collection`. Raft version missing: inherent `new()`, `Default`, `IntoIterator`.
**Fix:** Move to `core/` as the canonical `Collection` implementation. Add missing impls to raft first.

---

## 5. `EventLevel` / `RaftEventLevel` Enum

- `etheram/src/observer.rs` (`EventLevel`)
- `raft-node/src/observer.rs` (`RaftEventLevel`)

**Status:** Identical variants (`None, Essential, Info, Debug, Trace`). Protocol-agnostic.
**Fix:** Move to `core/src/event_level.rs`.

---

## 6. Generic I/O Adapter Traits (6 of 8)

- `TimerInputAdapter`, `TimerOutputAdapter`, `ExternalInterfaceIncomingAdapter`, `ExternalInterfaceOutgoingAdapter`, `TransportIncomingAdapter`, `TransportOutgoingAdapter`

**Status:** Identical "pin one associated type on a core trait" patterns in both `etheram/` and `raft-node/`. `StorageAdapter` and `CacheAdapter` are protocol-specific and stay separate.
**Fix:** Move the 6 generic adapter traits to `core/` alongside their base traits.
