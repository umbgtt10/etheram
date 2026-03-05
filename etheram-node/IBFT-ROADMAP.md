# Istanbul BFT — Feature Status

**Scope:** IBFT consensus protocol in EtheRAM
**Implementation:** `etheram-node/src/implementations/ibft/`
**Validation model:**
- Stage 1: protocol tests in `etheram-node/tests/implementations/ibft/`
- Stage 2: cluster tests in `etheram-validation/tests/`
- Stage 3: embedded end-to-end in `etheram-embassy/`

**Related:** [Chain Roadmap](CHAIN-ROADMAP.md) — Ethereum-like chain features

---

## Supported Features

### Core Consensus

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| Static validator set | Fixed set of `n` validators, `f = ⌊(n-1)/3⌋` | ✅ | ✅ | ✅ |
| Round-robin proposer | Deterministic proposer selection: `proposer = (height + round) % n` | ✅ | ✅ | ✅ |
| Three-phase commit | PrePrepare → Prepare → Commit flow | ✅ | ✅ | ✅ |
| Quorum computation | `⌊2n/3⌋ + 1` via `ValidatorSet` / `VoteTracker` | ✅ | ✅ | ✅ |
| Block proposal | Timer-driven `ProposeBlock` triggers block construction from pending transactions | ✅ | ✅ | ✅ |
| Block commit | Quorum of commit votes triggers `StoreBlock` + `IncrementHeight` + account updates | ✅ | ✅ | ✅ |
| Automatic round progression | Timer re-fires after commit to drive next height | ✅ | ✅ | ✅ |

### Safety and Robustness

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| View change | `TimeoutRound` → quorum of `ViewChange` → `NewView` → resume | ✅ | ✅ | ✅ |
| Locked-block preservation | `pending_block` not cleared on round change when `PreparedCertificate` is set | ✅ | ✅ | ✅ |
| Locked-block re-propose | Proposer with cert re-proposes the locked block, not a fresh block | ✅ | ✅ | ✅ |
| Highest-round cert wins | Incoming cert with higher round replaces current; lower or equal is ignored | ✅ | ✅ | ✅ |
| NewView authority | `valid_new_view` guard is the sole gate; no second compatibility check | ✅ | ✅ | ✅ |
| Message deduplication | Per-sender/per-kind duplicate filtering | ✅ | ✅ | — |
| Replay prevention | Sequence-based replay detection | ✅ | ✅ | — |
| Malicious block rejection | Conflicting PrePrepare from same proposer/round quarantines sender | ✅ | ✅ | — |
| Block validation | State root, account balance, nonce, gas-limit checks before voting | ✅ | ✅ | ✅ |
| Block re-execution | Validators re-execute transactions and compare `post_state_root` + `receipts_root` | ✅ | ✅ | ✅ |
| Future-round buffer | PrePrepare/Prepare/Commit for future rounds buffered; replayed on round advance | ✅ | — | — |

### Cryptographic Abstraction

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| `SignatureScheme` trait | Generic signing/verification interface with `SignatureBytes` newtype | ✅ | ✅ | ✅ |
| `MockSignatureScheme` | Zeroed sigs, always-true verify (deterministic testing) | ✅ | ✅ | ✅ (in-memory) |
| `Ed25519SignatureScheme` | Real `ed25519-dalek` signing and verification | ✅ | — | ✅ (real) |
| Commit signatures | `Commit` messages carry `sender_signature` verified via `commit_commitment_payload` | ✅ | ✅ | ✅ |
| PreparedCertificate verification | `valid_prepared_certificate` verifies each signature against canonical payload | ✅ | ✅ | ✅ |
| Injection-based testing | `AlternateSignatureScheme` + Byzantine cert forgery tests | ✅ | ✅ | — |

### Persistence and Recovery

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| `ConsensusWal` | Write-ahead log serialization (`to_bytes` / `from_bytes`) | ✅ | ✅ | ✅ |
| `WalWriter` trait | Abstraction for WAL persistence (in-memory, semihosting, etc.) | ✅ | — | ✅ |
| Restart recovery | `IbftProtocol::from_wal` restores state from WAL bytes | ✅ | ✅ | ✅ |
| `CapturedWalWriter` | Sticky `last_cert` capture that survives `reset_after_commit` | — | — | ✅ |

### Validator Set Management

| Feature | Description | Stage 1 | Stage 2 | Stage 3 |
|---|---|---|---|---|
| `ValidatorSetUpdate` | Height-gated scheduled transitions | ✅ | ✅ | ✅ |
| Graceful transition | Consensus continues through validator set change | ✅ | ✅ | ✅ |

---

## Planned Features

| Feature | Description | Priority | Complexity |
|---|---|---|---|
| Weighted voting | Stake-aware quorum (weighted `VoteTracker`) | Medium | Medium |
| Signature aggregation (BLS) | BLS `SignatureScheme` implementation; aggregate quorum proofs | Medium | High |
| Pipelined consensus | Overlap proposal of height `h+1` with commit of height `h` | Low | High |
| Optimistic responsiveness | Fast-path commit when all validators respond within a threshold | Low | Medium |
| Proposal buffering | Speculative execution of proposed blocks before consensus | Low | Medium |
| Dynamic validator discovery | Peer discovery without static validator list | Low | High |
| Byzantine evidence collection | Record and export evidence of Byzantine behavior | Medium | Medium |
| Slashing pipeline | Economic penalties for provable Byzantine actions | Low | High |
| Full crash-recovery simulation | Terminate mid-consensus → restart → confirm WAL recovery (Stage 3) | Medium | Medium |
| Malicious injection in QEMU | Byzantine fault injection in the embedded scenario | Low | Medium |

---

## Test Coverage Summary

| Area | Stage 1 (protocol) | Stage 2 (cluster) | Stage 3 (QEMU) |
|---|---|---|---|
| Proposal / PrePrepare / Prepare / Commit | ✅ | ✅ | ✅ (Act 0) |
| View change / NewView | ✅ | ✅ | ✅ (Act 4) |
| Client request/response | ✅ | ✅ | ✅ (Acts 1-6) |
| Deduplication / replay | ✅ | ✅ | — |
| Persistence / WAL | ✅ | ✅ | ✅ (Act 8) |
| Malicious blocks / quarantine | ✅ | ✅ | — |
| Ed25519 signatures | ✅ | — | ✅ (Act 9) |
| Validator set updates | ✅ | ✅ | ✅ (Act 7) |
| Block re-execution | ✅ | ✅ | ✅ |
| Future-round buffer | ✅ | — | — |
