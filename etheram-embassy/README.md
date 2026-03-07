# etheram-embassy

> `no_std` + Embassy embedded port — 5-node IBFT on ARM Cortex-M4

`no_std` + Embassy port of the EtheRAM node. Runs a five-node IBFT cluster on ARM
Cortex-M4 (QEMU `mps2-an386`) across two independently feature-gated hardware
configurations. Validates the full 3-6 architectural model under embedded constraints:
Byzantine consensus, real Ed25519 signatures, durable WAL, and async task isolation —
all without the standard library.

**Parent:** [EtheRAM](../README.md)
**Depends on:** [core](../core/README.md), [embassy-core](../embassy-core/README.md), [etheram-node](../etheram-node/README.md)

---

## Canonical Project Docs

| Document | Scope |
|---|---|
| [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) | Stable 3-6 model and execution semantics |
| [../docs/IMPLEMENTED-CAPABILITIES.md](../docs/IMPLEMENTED-CAPABILITIES.md) | Workspace-level implemented capability matrix |
| [../docs/ROADMAP.md](../docs/ROADMAP.md) | Remaining feature families and added project value |

---

## Purpose

This crate answers one question: **does the EtheRAM architecture hold under embedded
constraints?**

Concretely it must demonstrate:

- Five concurrent IBFT validator nodes running as independent Embassy async tasks on a
  single Cortex-M4, communicating only through their declared I/O dimensions.
- Full protocol coverage in QEMU: multi-round consensus, view change, transaction
  validation, validator set transitions, WAL crash-recovery, and cryptographic signature
  verification.
- Two independently viable hardware configurations — all-in-memory (development) and real
  (production-proxy with UDP sockets, semihosting storage, and Ed25519 keys) — that are
  both always-green and never cross-contaminate each other.

The crate is not a production node. It is a correctness and architecture proof under
resource constraints.

---

## Implemented Embedded Surface

### Two always-green configurations

Both must compile, link, and execute to `All tests passed` at all times. Neither can be
broken while working on the other.

| Feature combination | Transport | Storage | Signature scheme | Script |
|---|---|---|---|---|
| `channel-transport` + `in-memory-storage` + `channel-external-interface` | Embassy channel arrays | `InMemoryStorage` | `MockSignatureScheme` | `run_channel_in_memory.ps1` |
| `udp-transport` + `semihosting-storage` + `udp-external-interface` | `embassy-net` UDP sockets over `MockNetDriver` | `SemihostingStorage` | `Ed25519SignatureScheme` | `run_udp_semihosting.ps1` |

Mutual-exclusivity guards in `src/configurations/mod.rs` produce a `compile_error!` if
conflicting features are combined.

### Async per-node task isolation

Each node runs as a `#[embassy_executor::task(pool_size = 5)]`. The task loops on
`select4` across four futures: cancellation token, inbound transport message, timer command,
and external interface notification. Protocol steps are synchronous (`node.step()`) inside
the async loop — no I/O occurs inside the protocol.

`OutboxTransport` bridges the sync protocol write path to async channel sends: the protocol
pushes outgoing messages into a `Vec` buffer; the task drains that buffer into the Embassy
channel senders after each step cycle.

### Real UDP transport (in-memory NIC)

The real configuration uses genuine `embassy-net` UDP sockets. Each node gets a dedicated
network stack on a `MockNetDriver` that routes Ethernet frames between the five stacks via
static `Channel` arrays in `NetworkBus` — a NIC simulation at the driver boundary, not at
the socket boundary. Each node binds `UdpSocket` to `10.0.0.(index+1):900(index)`. Messages
are serialized with `postcard` as `Envelope { from, message_bytes }`.

The socket send/receive path is identical to what would run against a physical NIC; only
the driver underneath differs.

### Durable WAL via ARM semihosting

`SemihostingStorage` persists consensus state through ARM semihosting file I/O. On startup
the node attempts to restore its `IbftProtocol` from WAL before falling back to cold start.
`CapturedWalWriter` maintains a sticky `last_cert` that survives `reset_after_commit`,
enabling post-commit inspection of the last `PreparedCertificate` without re-running the
protocol.

### Ed25519 cryptographic signatures

The real configuration wires `Ed25519SignatureScheme` unconditionally. Five hardcoded
Cortex-M4 keypairs (one per node index) are embedded as const byte arrays. Every Prepare
message carries a `sender_signature: SignatureBytes`; quorum is not declared until
`valid_prepared_certificate` verifies the required number of Ed25519 signatures against the
canonical prepare-commitment payload. A forged cert with zeroed signatures fails this check
and is silently dropped.

The in-memory configuration retains `MockSignatureScheme` (always-true verify, zeroed sigs)
to keep development fast and deterministic.

### 12-act QEMU scenario

`main.rs` runs a structured lifecycle that exercises every major protocol path:

| Act | What it proves |
|---|---|
| 0 — Warmup | Multi-round consensus; automatic timer re-fire after each commit; height advances to ≥ 10 |
| 1–2 — Transfers | `ClientRequest` submission end-to-end; balance mutations committed and queryable |
| 3 — Overdraft | `InsufficientBalance` rejection before block inclusion |
| 4 — View change | `TimeoutRound` quorum triggers round rotation; consensus resumes |
| 5 — Stale nonce | `InvalidNonce` rejection |
| 6 — Gas limit | `GasLimitExceeded` rejection |
| 7 — Validator set update | Height-gated transition at height 5; consensus continues with updated validator set |
| 8 — WAL round-trip | `ConsensusWal::to_bytes` → `from_bytes` verified in-process |
| 9 — Ed25519 cert proof | `PreparedCertificate.signed_prepares` count equals quorum; real config sigs are non-zero |
| 10 — TinyEVM SSTORE | Contract storage demonstration; SSTORE opcode persists key-value in contract storage |
| 11 — TinyEVM OutOfGas | Gas failure demonstration; transaction with insufficient gas reverts with OutOfGas |

---

## Architecture

### Node step loop

Each node follows the EtheRAM 3-6 model on every `step()` call:

1. Poll incoming sources (timer, external interface, transport) for the next event.
2. Build context from current state via `EagerContextBuilder`.
3. Pass event to `IbftProtocol::handle_message()` → list of `Action<IbftMessage>`.
4. `TypeBasedPartitioner` splits actions into state mutations, output effects, and block executions.
5. Mutations applied to storage/cache. Output effects dispatched by `EtheramExecutor`.
6. Block executions trigger `ExecutionEngine` for transaction processing, receipts, and contract storage updates.

No I/O occurs inside the protocol. The executor handles all side effects after the protocol
returns.

### Dimension coverage

| Dimension | In-memory config | Real config |
|---|---|---|
| Protocol | `IbftProtocol<MockSignatureScheme>` | `IbftProtocol<Ed25519SignatureScheme>` |
| Storage | `InMemoryStorage` | `SemihostingStorage` |
| Cache | `InMemoryCache` | `InMemoryCache` |
| Transport | `ChannelTransportHub` / `OutboxTransport` | `embassy-net` UDP / `UdpTransportHub` |
| External Interface | `ChannelExternalInterface` | `UdpExternalInterface` |
| Timer | `InMemoryTimer` | `InMemoryTimer` |

### Infrastructure axes

Each axis is independently feature-gated under `src/infra/`:

```
src/infra/
  external_interface/
    channel/          ← channel-external-interface
    udp/              ← udp-external-interface
  storage/
    in_memory/        ← in-memory-storage
    semihosting/      ← semihosting-storage
  transport/
    channel/          ← channel-transport
    udp/              ← udp-transport
```

Adding a new variant means adding a subfolder under the relevant axis and updating only
that axis's `mod.rs`. Wiring happens in `src/configurations/in_memory/setup.rs` and
`src/configurations/real/setup.rs`.

### Source layout

```
src/
  main.rs                       # lifecycle: init → start → 12-act scenario → shutdown
  config.rs                     # MAX_NODES = 5
  cancellation_token.rs
  etheram_client.rs             # submit_request, node_height, node_wal, node_last_cert, shutdown
  spawned_node.rs               # per-node shared state (height, wal, last_cert, timer channel)
  time_driver.rs                # SystickDriver (ARM SysTick ISR at 1 ms)
  logging.rs                    # SemihostingWriter + info! macro
  heap.rs                       # static allocator
  semihosting_observer.rs       # Observer impl that logs via semihosting
  infra/                        # three independently feature-gated axes
    external_interface/
      channel/
      udp/
    storage/
      in_memory/
      semihosting/
      captured_wal_writer.rs    # sticky last_cert capture
    transport/
      channel/
      udp/
  configurations/
    mod.rs                      # mutual-exclusivity compile_error! guards
    in_memory/setup.rs
    real/setup.rs
```

### Hardcoded node count

`MAX_NODES = 5` sizes all static arrays. Changing the cluster requires updating this
constant and recompiling. Two evolutionary paths exist:

- **Option A** — keep `MAX_NODES` as the static ceiling; introduce a separate runtime
  `VALIDATOR_COUNT`. Arrays stay sized to `MAX_NODES`; only the first `VALIDATOR_COUNT`
  slots are used.
- **Option B** — const-generic infrastructure types parameterized on `N`. More idiomatic
  but requires propagating the parameter through every hub and task pool.

### QEMU vs production

| Aspect | Current (QEMU single-process) | Production |
|---|---|---|
| Node isolation | 5 tasks, one address space | 5 separate processes or machines |
| Transport | Channels or `embassy-net` over `MockNetDriver` | Real UDP/TCP over hardware NIC |
| Signature scheme | Mock (in-memory) / Ed25519 (real) | Ed25519 / BLS |
| Storage | In-memory or semihosting file | Flash / RocksDB |
| Timing | SysTick ISR at 1 ms | NTP wall clock |
| External clients | Channel hub inside same process | TCP / IPC boundary |

---

## Open Points

- **Multi-process / multi-machine deployment** — all five nodes share one QEMU address
  space. Real network partitions, clock skew, and Byzantine fault injection at the OS
  boundary require separate processes or separate boards communicating over physical UDP.
  This is the only remaining gap between the current proof and a production deployment
  model.
