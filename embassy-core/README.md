# embassy-core

> Shared `no_std` Embassy infrastructure for protocol-family embedded crates

`embassy-core` contains the protocol-agnostic embedded building blocks reused by `etheram-embassy` and `raft-embassy`. It centralizes the Embassy-specific machinery that should not be duplicated per protocol family: transport hubs, external-interface/client-facade macros, network helpers, timer channels, logging, heap setup, shared state, and cancellation support.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md)
**Used by:** [etheram-embassy](../etheram-embassy/README.md), [raft-embassy](../raft-embassy/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate exists to keep embedded infrastructure reuse explicit. Without it, the Etheram and Raft Embassy crates would drift into duplicated runtime wiring even though their protocol logic is intentionally independent.

`embassy-core` is therefore not a protocol crate. It is the shared embedded substrate that proves the two protocol families can share runtime mechanics without sharing protocol code.

---

## Implemented Infrastructure Surface

- cancellation token support for coordinated task shutdown
- channel transport hub and outbox bridge helpers
- client request hub and client facade macros
- network bus and network configuration helpers
- timer channels and time-driver support
- embedded heap, logging, and shared-state helpers

These modules are exported from `src/lib.rs` and consumed by the Embassy-facing protocol crates.

---

## Source Layout

```
src/
  cancellation_token.rs
  channel_transport_hub.rs
  channel_external_interface_macro.rs
  client_facade_macro.rs
  client_request_hub.rs
  network_bus.rs
  network_config.rs
  outbox_transport.rs
  timer_channels.rs
  time_driver.rs
  embassy_shared_state.rs
  heap.rs
  logging.rs
  config.rs
  lib.rs
```

---

## Why This Crate Matters

`embassy-core` makes the embedded story more than two parallel one-off ports. It captures the reusable runtime layer shared by the protocol families while preserving the rule that Etheram and Raft depend only on `core/` for protocol semantics and never on each other.
