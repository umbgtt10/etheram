# EtheRAM — Remaining Roadmap

This document is the canonical list of work that is not yet implemented. It complements [ARCHITECTURE.md](ARCHITECTURE.md), which explains the stable model, and [IMPLEMENTED-CAPABILITIES.md](IMPLEMENTED-CAPABILITIES.md), which records what is already proven.

Implemented milestones are intentionally omitted here.

---

## Remaining Feature Families

| Feature Family | Remaining Features | Additional Value To Project | Relative Effort |
|---|---|---|---|
| IBFT extensions | Weighted voting; BLS signature aggregation; pipelined consensus; optimistic responsiveness; proposal buffering; dynamic validator discovery; Byzantine evidence collection; slashing pipeline; embedded crash-recovery simulation; embedded Byzantine injection | Extends the Byzantine story from a solid research node into a more realistic BFT platform, especially by deepening crypto swappability and adversarial runtime validation | Medium-High |
| Ethereum execution and chain depth | Merkle Patricia Trie state root; full EVM opcode coverage; contract deployment via `CREATE`/`CREATE2`; EIP-2930 access lists; transaction fees; state snapshots; JSON-RPC interface; production-style storage/transport/timer variants where relevant | Moves EtheRAM from “Ethereum-like and architecturally credible” toward “tool-compatible and execution-complete,” which would materially broaden external relevance | High |
| Raft extensions | Joint-consensus configuration changes; read leases / `ReadIndex`; chunked snapshot transfer; leader lease; batched replication; log compaction trigger; real snapshot storage; embedded fault injection | Completes the Raft side as a serious peer to the IBFT family rather than a second-but-narrower proof of generality | Medium |
| Third protocol family | HotStuff, Multi-Paxos, or Tendermint as a new independent crate family | Would be the strongest possible evidence that the 3-6 model is genuinely general rather than accidentally fitting exactly two protocol families | High |
| Formal methods on Rust artifacts | Kani model checking for protocol invariants on the Rust handlers | Adds machine-checked evidence directly on the implementation, strengthening the research argument beyond tests and TLA+ alone | Medium |
| Production infrastructure | libp2p transport; RocksDB storage; Docker cluster deployment; throughput and overhead benchmarks | Answers the practical question of whether the decomposition remains viable under real operational and performance pressures | Medium-High |
| Tooling and observability | Protocol visualizer; deterministic replay debugger; chaos transport/testing framework | Converts architectural cleanliness into distinctive developer and demo tooling, making the framework easier to explain, debug, and evaluate | Low-Medium |
| Physical deployment | Real hardware deployment on STM32 / RP2040 and comparable targets | Demonstrates that the `no_std` story is not just QEMU-valid but hardware-valid, which materially increases credibility for embedded claims | High |

---

## Recommended Reading

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for the stable model.
2. Read [IMPLEMENTED-CAPABILITIES.md](IMPLEMENTED-CAPABILITIES.md) for what the project already proves.
3. Use this document to evaluate which remaining family adds the most leverage next.