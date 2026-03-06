# Etheram Node Process - Sync Implementation Plan

## Goal
Implement a clean block synchronization protocol in `etheram-node-process` so lagging nodes recover after partition/heal without transport replay workarounds.

## Scope
- Add sync control plane and messages in node-process transport/runtime layers.
- Keep IBFT consensus logic in `etheram-node` unchanged and pure.
- Use deterministic block import validation before applying synced blocks.

## Non-goals
- No packet replay buffering as a recovery strategy.
- No protocol-specific catch-up logic inside IBFT message handlers.
- No embassy changes in this phase.

## Architecture Boundary
- `etheram-node`: consensus and block validation primitives.
- `etheram-node-process`: sync orchestration, peer status exchange, lag detection, range requests, retry policy.

## Milestones

### M1. Sync Message Types and Wire Mapping
- Add sync messages to process transport payloads:
  - `Status { height, last_hash }`
  - `GetBlocks { from_height, max_blocks }`
  - `Blocks { start_height, blocks }`
- Add serialization/deserialization and dispatch routing.

Acceptance criteria:
- Nodes can send/receive sync messages over gRPC transport.
- Existing IBFT peer traffic remains unaffected.

### M2. Status Gossip and Lag Detection
- Periodically emit local `Status` to peers.
- Track highest observed peer height.
- Enter `NeedSync` when local node is behind by threshold >= 1 block.

Acceptance criteria:
- Lagging node reports sync-needed state in logs.
- No false-positive sync loops when node is at tip.

### M3. Block Range Request/Response
- Request missing finalized blocks with bounded `max_blocks`.
- Respond from local storage for requested range.
- Support retry and peer failover when a request times out.

Acceptance criteria:
- Lagging node receives contiguous ranges until tip.
- Requests are bounded and backpressure-safe.

### M4. Deterministic Import Pipeline
- Validate parent linkage and block commitments.
- Import in strict height order.
- Abort and re-request on validation failure.

Acceptance criteria:
- Imported height increases monotonically.
- Invalid ranges are rejected safely.

### M5. Rejoin Normal Consensus
- Exit sync state at tip.
- Resume normal IBFT participation automatically.

Acceptance criteria:
- Post-sync node converges with peers in height/hash.

## Mandatory Programmatic Partition Tests
These tests are required during implementation, not deferred.

1. Partition-and-heal lag recovery test
- 5 nodes.
- Apply one-way partition `1 -> 4`.
- Let node 4 fall behind.
- Heal link.
- Assert node 4 catches up to cluster tip without replay buffering.

2. Long partition test
- Keep partition long enough to create a large height gap.
- Assert multi-batch sync catches up fully.

3. Invalid range response test
- Inject malformed block range.
- Assert node rejects invalid data and retries/fails over.

4. Peer failover test
- Active sync peer goes offline mid-sync.
- Assert node switches peer and finishes sync.

5. No-op sync test
- All nodes already at tip.
- Assert no unnecessary block requests.

## Operational Constraints
- Keep queue sizes bounded.
- Use retry backoff and timeout limits.
- Log every sync state transition for observability.

## Completion Gate
For each productive change batch:
- `powershell -File scripts/run_tests.ps1`
- `cargo check -p etheram-core --no-default-features`
- `cargo check -p embassy-core --no-default-features`
- `cargo check -p etheram-node --no-default-features`
- `cargo check -p raft-node --no-default-features`

Direction H fast-path applies: `scripts/run_apps.ps1` is not required unless changes touch embassy crates.
