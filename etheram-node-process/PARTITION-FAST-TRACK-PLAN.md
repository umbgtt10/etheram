# Etheram Node Process - Fast Track Plan for Network Partition Testing

## Goal
Reach the earliest possible point where runtime `partition/heal` actions can be executed against a multi-process cluster and their impact on consensus progress can be observed.

## Scope Strategy
Prioritize a thin vertical slice focused on partition behavior.

In scope now:
- Process bootstrap + long-running loop
- Observable node status logs
- Peer transport over gRPC (minimal surface)
- Partitionable transport decorator
- Runtime control API for `partition/heal`
- Minimal launcher command surface

Deferred until after partition testing works:
- Sled storage
- Full desktop dashboard UI
- Client gRPC external interface
- WAL-backed crash recovery

## Milestones

### M1. Runtime Observability Baseline (started)
- `etheram-node-process` supports long-running execution mode
- Periodic status output includes:
  - `peer_id`
  - current chain `height`
  - last committed block hash (short hex)
  - executed step counters
- CLI keeps finite-step mode for deterministic tests

Exit criteria:
- Process can run continuously and emit stable heartbeat/status lines.

### M2. Minimal Peer gRPC Transport
- Add `grpc_transport/` with:
  - `GrpcTransportIncoming`
  - `GrpcTransportOutgoing`
- Support unary send for peer message ingress
- Broadcast implemented as local fan-out over peer clients

Exit criteria:
- Two local node processes can exchange peer messages via gRPC.

### M3. Partitionable Transport Decorator
- Add `partitionable_transport/` wrapper around transport outgoing path
- Keep in-memory blocked-link table:
  - `BTreeSet<(PeerId, PeerId)>`
- Drop blocked sends and emit partition-drop logs

Exit criteria:
- At runtime, blocking a link causes deterministic send drops.

### M4. Partition Control Plane
- Add minimal control endpoint on each node process
- Commands:
  - `partition <a> <b>`
  - `heal <a> <b>`
  - `clear` (optional)
- Apply updates to partition table without process restart

Exit criteria:
- Operator can mutate partition state live.

### M5. Minimal Launcher Integration
- `etheram-desktop` can spawn N node processes from `cluster.toml`
- Pass through partition/heal commands to target process controls
- Merge and display process stdout with node labels

Exit criteria:
- One command path from launcher to partition controls is working.

### M6. Partition Scenario Test
- Start 5 processes
- Wait for baseline height growth
- Partition critical links
- Verify height progression stalls/degrades as expected
- Heal links
- Verify progress resumes

Exit criteria:
- Repeatable pass of partition/heal scenario with observable before/after behavior.

## Immediate Next Coding Steps
1. Complete M1 in `etheram-node-process` runtime loop and status printing.
2. Add one integration test for finite loop mode stability.
3. Begin M2 with gRPC transport module scaffolding and compile-only wiring.
