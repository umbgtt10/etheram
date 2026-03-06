# Etheram Node Process - Sync Implementation Status

## Goal
Deliver clean sync orchestration in `etheram-node-process` so lagging nodes recover after partition/heal without transport replay workarounds.

## Scope
- Sync control plane and message routing in process transport/runtime layers.
- Keep IBFT protocol logic in `etheram-node` unchanged and pure.
- Deterministic import validation before applying synchronized blocks.

## Architecture Boundary
- `etheram-node`: consensus protocol and block semantics.
- `etheram-node-process`: status gossip, lag detection, range planning, retry/failover, import orchestration.

## Implemented Milestones

### M1. Sync Message Types and Wire Mapping
- Implemented `SyncMessage::{Status, GetBlocks, Blocks}`.
- Implemented wire serialization/deserialization and sync-vs-IBFT dispatch.
- Covered with integration tests in:
  - `tests/wire_node_message_tests.rs`
  - `tests/grpc_transport_tests.rs`
  - `tests/sync_sender_tests.rs`

### M2. Status Gossip and Lag Detection
- Periodic status gossip emitted from runtime loop.
- `SyncState` tracks observed heights and lag distance.
- No-op behavior at tip covered by tests.

### M3. Block Range Request/Response + Failover
- Bounded requests (`SYNC_MAX_BLOCKS_PER_REQUEST`) and range responses.
- In-flight request tracking implemented.
- Timeout/retry/backoff policy implemented with retry budget and peer failover.
- Failover and timeout behavior covered in `tests/sync_state_tests.rs` and `tests/sync_plan_mandatory_tests.rs`.

### M4. Deterministic Import Pipeline
- Strict start-height and contiguous height checks.
- Parent linkage check for first imported block via expected parent `post_state_root`.
- Commitment checks:
  - block gas limit must be canonical and non-zero
  - sum(tx gas_limit) must not exceed block gas limit
- Invalid payload/range handling rejects safely and drives failover logic.

### M5. Rejoin Normal Consensus
- Sync apply path advances storage height monotonically and re-enters normal request planning at tip.
- Runtime/state-level convergence and no-op-at-tip flows covered in sync runtime/mandatory tests.

## Mandatory Programmatic Scenario Tests

Implemented in `tests/sync_plan_mandatory_tests.rs`:
1. `partition_and_heal_lag_recovery_selects_new_request_and_imports_after_heal`
2. `long_partition_multi_batch_sync_import_catches_up_fully`
3. `invalid_range_response_is_rejected_and_failover_is_planned`
4. `active_sync_peer_offline_mid_sync_switches_peer_and_completes`
5. `no_op_sync_at_tip_plans_no_request`

## Operational Constraints Implemented
- Bounded request size.
- Explicit request timeout + retry budget + failover behavior.
- Sync state transitions logged via runtime sync log lines.

## Completion Gate
For productive sync changes:
- `powershell -File scripts/run_tests.ps1`
- `cargo check -p etheram-core --no-default-features`
- `cargo check -p embassy-core --no-default-features`
- `cargo check -p etheram-node --no-default-features`
- `cargo check -p raft-node --no-default-features`
- `powershell -File scripts/run_apps.ps1` when full app verification is requested
