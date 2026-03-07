# etheram-desktop

> Native launcher and dashboard for the Direction H multi-process cluster

`etheram-desktop` is the operator-facing desktop layer for the EtheRAM multi-process cluster. It reads `cluster.toml`, spawns one `etheram-node-process` child per node, relays control commands, and provides both CLI and GUI entry points.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [etheram-node](../etheram-node/README.md), [etheram-node-process](../etheram-node-process/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate completes Direction H by turning the desktop cluster into something that can be launched, observed, and controlled as a native application rather than only through tests and scripts.

It is explicitly `std`-only and is not part of the `no_std` story.

---

## Entry Points

### CLI mode

```text
etheram-desktop <cluster.toml>
```

CLI mode starts the fleet and then accepts live commands from stdin through the launcher.

### GUI mode

```text
etheram-desktop --gui [cluster.toml]
```

GUI mode starts the native dashboard built on `eframe`.

---

## Launcher

`src/launcher.rs` is the operational core.

It is responsible for:

- spawning child `etheram-node-process` instances
- wiring stdout log pumping per child
- broadcasting partition and heal commands to the fleet
- issuing coordinated shutdown
- supporting finite-step runs through environment configuration for tests and demos

Supported live commands:

- `partition <from> <to>`
- `heal <from> <to>`
- `clear`
- `shutdown`

---

## Dashboard

The UI under `src/ui/` is fed by launcher state and child-process output rather than by direct node embedding.

That preserves the architectural point of Direction H: nodes remain isolated OS processes while the desktop layer acts as launcher and observer.

---

## Relationship To Node Process

`etheram-desktop` does not implement consensus logic. It depends on `etheram-node-process` to host each validator and uses `cluster.toml` as the fleet-level source of truth.

This split gives the desktop path:

- real process isolation
- real gRPC networking between nodes
- real per-node durable state and WAL recovery
- live operational control from a single desktop application

---

## Tests

The integration suite covers launcher startup and launcher behavior through `tests/all_tests.rs`, including child-process orchestration paths used by the desktop fleet.
