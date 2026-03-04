# core

> Foundational traits for distributed node decomposition

The core crate defines the minimal, `no_std`-compatible trait surface that all EtheRAM crates build on. It provides the behavioral building blocks — dimension I/O traits, the consensus protocol interface, and the node execution primitive — without prescribing how they are composed.

**Parent:** [EtheRAM](../README.md)
**Dependents:** [etheram](../etheram/README.md) → [etheram-variants](../etheram-variants/README.md) → [etheram-validation](../etheram-validation/README.md), [etheram-embassy](../etheram-embassy/README.md)

---

## Constraints

- `#![no_std]` — no standard library dependency
- No `alloc` — core uses only stack-allocated types and references
- Traits define behavior, not structure — consumers compose them freely

---

## Contents

### Dimension I/O Traits

Each of the six architectural dimensions is split into input and output halves:

| Trait | Dimension | Direction | Key method |
|---|---|---|---|
| `TimerInput` | Timer | In | `poll() → Option<Event>` |
| `TimerOutput` | Timer | Out | `schedule(event, delay)` |
| `TransportIncoming` | Transport | In | `poll() → Option<(PeerId, Message)>` |
| `TransportOutgoing` | Transport | Out | `send(peer_id, message)`, `broadcast(message)` |
| `ExternalInterfaceIncoming` | ExternalInterface | In | `poll() → Option<Request>` |
| `ExternalInterfaceOutgoing` | ExternalInterface | Out | `respond(response)` |
| `Storage` | Storage | Read/Write | `query()`, `apply_mutation()` |
| `Cache` | Cache | Read/Write | `query()`, `apply_mutation()` |

All traits use associated types for protocol-specific messages, events, and queries. No trait prescribes a specific data format.

### Consensus Protocol Trait

```rust
pub trait ConsensusProtocol {
    type Message;
    type MessageSource;
    type Action;
    type Context;
    type ActionCollection: Collection<Item = Self::Action>;

    fn handle_message(
        &self,
        source: &Self::MessageSource,
        message: &Self::Message,
        ctx: &Self::Context,
    ) -> Self::ActionCollection;
}
```

Pure function: immutable input, declarative output. No I/O, no side effects. This is the Brain Space interface.

### Node Trait

```rust
pub trait Node {
    type Id: Copy;
    fn step(&mut self) -> bool;
    fn id(&self) -> Self::Id;
}
```

Minimal execution primitive for multi-node orchestration. `step()` returns `true` if work was done.

### Supporting Types

| Type | Purpose |
|---|---|
| `PeerId` | `u64` node identifier |
| `Collection` | Trait for iterable action containers (enables `no_std` action collections) |

---

## Source Layout

```
src/
  lib.rs                          #![no_std], pub mod declarations only
  types.rs                        PeerId
  consensus_protocol.rs           ConsensusProtocol trait
  node.rs                         Node trait
  collection.rs                   Collection trait
  timer_input.rs                  TimerInput trait
  timer_output.rs                 TimerOutput trait
  transport_incoming.rs           TransportIncoming trait
  transport_outgoing.rs           TransportOutgoing trait
  external_interface_incoming.rs  ExternalInterfaceIncoming trait
  external_interface_outgoing.rs  ExternalInterfaceOutgoing trait
  storage.rs                      Storage trait
  cache.rs                        Cache trait
```

---

## Design Rationale

Core was originally a prescriptive 6-dimension Node trait with all associated types. That was removed because it dictated HOW (structure), not just WHAT (behavior). The current design provides composable building blocks: consumers wire them into whatever node structure fits their needs.

See [ADR-001](../docs/ADR/001-six-dimension-node-decomposition.md) for the full rationale.
