# ADR-003: embassy-core Extraction and Dual-Configuration Strategy

## Status

**Accepted** — Implemented across etheram-embassy and raft-embassy (March 2026)

## Context

EtheRAM supports deployment on ARM Cortex-M4 embedded hardware (QEMU and physical) via two Embassy async runtimes: `etheram-embassy` and `raft-embassy`. Both crates require identical infrastructure to operate in a no_std Embassy environment:

- A **cancellation primitive** to coordinate graceful shutdown across concurrent async node tasks
- An **async-compatible shared state** wrapper around `spin::Mutex` for cross-task data sharing
- An **embedded heap allocator** to enable dynamic allocation on Cortex-M4
- A **semihosting logger** backed by `cortex-m-semihosting` for QEMU-visible output
- A **SysTick time driver** implementing `embassy-time-driver` for timer scheduling
- A **sync-to-async transport bridge** (`OutboxTransport`) for injecting synchronous `send()` results into async Embassy channels
- **Embassy channel infrastructure** for in-process transport and external interface communication
- **UDP networking primitives** for real hardware transport and external interface

Without a shared location, every primitive above would be duplicated word-for-word across both embassy crates. Each update would require synchronised edits in two places, creating a silent drift risk that would be invisible to the compiler.

Additionally, each embassy crate must support at least two working deployment configurations with different infrastructure choices, validated simultaneously. The criteria for choosing infrastructure variants are non-obvious and have caused integration failures when one configuration was updated without the other.

## Decision

### A — Extract `embassy-core` as a Shared Infrastructure Crate

A dedicated `embassy-core` crate holds all Embassy infrastructure primitives shared between `etheram-embassy` and `raft-embassy`. Any primitive that is not protocol-specific belongs here.

**Current `embassy-core` inventory:**

| Module | Purpose |
|---|---|
| `cancellation_token` | `CancellationToken` — graceful shutdown signal for async node tasks |
| `embassy_shared_state` | `EmbassySharedState<T>` — `spin::Mutex`-based shared state compatible with Embassy async tasks |
| `heap` | `init_heap()` — Cortex-M4 heap initialisation via `embedded-alloc` |
| `logging` | `info!` macro backed by `cortex-m-semihosting::hprintln!` for QEMU output |
| `time_driver` | `SystickDriver` — SysTick exception handler implementing `embassy-time-driver` |
| `config` | Shared constants (`MAX_NODES`) |
| `network_bus` / `network_config` | UDP socket infrastructure with per-node port assignment |
| `outbox_transport` | `OutboxTransport` — injects synchronous outbound messages into Embassy async channels |
| `channel_transport_hub` | `ChannelTransportHub` — Embassy static channel arrays backing in-memory transport |
| `client_request_hub` | `ClientChannelHub` / `EtheramClient` — channel-based external interface |
| `timer_channels` | `TimerChannels` — static Embassy channel pool for timer events |
| `channel_external_interface_macro` / `client_facade_macro` | Shared proc-macro-free code generation helpers |

Both `etheram-embassy` and `raft-embassy` re-export types from `embassy-core` that their consumers need at the crate boundary:

```rust
pub use embassy_core::cancellation_token::CancellationToken;
```

### B — Each Embassy Crate Maintains Exactly Two Configurations

Every embassy crate defines exactly two end-to-end working configurations, enforced at compile time:

| Configuration | Transport | Storage | External Interface | Feature flags |
|---|---|---|---|---|
| **all-in-memory** | `ChannelTransportHub` | In-memory | `ChannelExternalInterface` | `channel-transport` + `in-memory-storage` + `channel-external-interface` |
| **real** | UDP (`UdpTransport`) | Semihosting file I/O | UDP (`UdpExternalInterface`) | `udp-transport` + `semihosting-storage` + `udp-external-interface` |

**Enforcement:** Each axis (transport, storage, external-interface) has a mutual-exclusivity `compile_error!` guard in `configurations/mod.rs`. Selecting two variants from the same axis is a compile-time error.

**Invariant:** Both configurations must compile, link, and produce correct QEMU output at all times. A change that fixes one configuration while breaking the other is rejected. The CI gate validates both feature combinations explicitly.

**Rationale for two configurations:** The all-in-memory configuration provides deterministic, repeatable QEMU runs with no external I/O. The real configuration validates the full stack including serialization (`postcard` / `WireIbftMessage` / `WireRaftMessage`), UDP socket binding, and semihosting file persistence. Each configuration exercises code paths the other does not.

## Consequences

### Positive

1. **No Duplication** — `CancellationToken`, `EmbassySharedState`, `heap`, and all shared Embassy primitives exist in exactly one location. A fix or enhancement applies to both protocol families automatically.
2. **Protocol Family Independence** — `etheram-embassy` and `raft-embassy` depend on `embassy-core` but not on each other. The cross-family dependency ban is preserved.
3. **Forced Dual Validation** — The two-configuration invariant prevents silent partial regression. Both feature sets are gate-tested on every CI run.
4. **Infrastructure Swappability** — The dual-configuration strategy is a live demonstration of the six-dimension decomposition: the node protocol is unchanged across both configurations; only infrastructure dimensions are swapped.
5. **Compile-Time Safety** — Mutual-exclusivity guards eliminate the class of bugs where two incompatible variants are selected simultaneously.
6. **Embedded Development Velocity** — Common Embassy primitives (heap, logging, time driver) are bootstrapped once in `embassy-core` and available to any future embassy crate without re-implementation.

### Negative

1. **Additional Crate Boundary** — Consumers of both `etheram-embassy` and the shared types must import from `embassy-core` directly or rely on the re-export; this requires knowing the re-export exists.
2. **Feature Flag Complexity** — Mutual-exclusivity is enforced at compile time but not by Cargo's feature system natively; `compile_error!` guards are necessary but unusual.
3. **Two-Configuration Maintenance Cost** — Every infrastructure change must be reflected in both configurations, which doubles the validation effort for axis-spanning changes.

### Validation Evidence

**`embassy-core` extraction validated by:**
- Both `etheram-embassy` configurations compile and execute a 12-act IBFT scenario end-to-end on QEMU
- Both `raft-embassy` configurations compile and execute a 5-act Raft scenario end-to-end on QEMU
- All four QEMU runs are part of the mandatory gate (`scripts/test.ps1`) and must pass before any change is merged

**Dual-configuration invariant validated by:**
- `run_channel_in_memory.ps1` and `run_udp_semihosting.ps1` (etheram-embassy) both run in CI
- `run_raft_channel_in_memory.ps1` and `run_raft_udp_semihosting.ps1` (raft-embassy) both run in CI
- Mutual-exclusivity guards verified via `cargo check` with conflicting feature combinations during development

## Related

- [ADR-001: Six-Dimension Node Decomposition](001-six-dimension-node-decomposition.md)
- [ADR-002: step() as Single Execution Primitive](002-step-as-single-execution-primitive.md)
- [Architecture Documentation](../ARCHITECTURE.md)
