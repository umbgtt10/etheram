# Istanbul BFT Roadmap (Trimmed)

**Purpose:** concise status of IBFT in EtheRAM
**Target:** Ethereum-like Byzantine Fault Tolerant consensus
**Validation model:**
- Stage 1: protocol tests in `etheram-variants/tests/implementations/ibft/`
- Stage 2: cluster tests in `etheram-validation/tests/`
- Stage 3: embedded compatibility in `etheram-embassy/`

---

## Implemented Features

### Core consensus
- Static validator set
- Round-robin proposer selection
- Three-phase flow: `PrePrepare` → `Prepare` → `Commit`
- Quorum computation from validator set (`2f+1` via current `ValidatorSet`/`VoteTracker` semantics)
- Leader block proposal and commit progression

### Safety and robustness (Phase 2 complete)
- View change + new view
- Replay prevention with per-sender/per-kind sequences
- Block validation before voting (state root, account checks, nonce, gas constraints)
- Persistent consensus state (WAL) and restart recovery
- Message deduplication
- Malicious conflicting block rejection + malicious sender quarantine behavior
- Graceful validator set updates (height-gated scheduled transitions)

### Crypto abstraction
- `SignatureScheme` trait with `SignatureBytes` newtype (`[u8; 96]`) integrated across the full protocol flow
- Every `Prepare` message carries a `sender_signature`; `PreparedCertificate` stores `signed_prepares: Vec<(PeerId, SignatureBytes)>`
- `valid_prepared_certificate` verifies each signature against the canonical prepare-commitment payload before accepting a cert on `ViewChange` / `NewView` — closes the Byzantine cert forgery attack vector
- `MockSignatureScheme` (zeroed sigs, always-true verify) for deterministic testing
- `Ed25519SignatureScheme` (real `ed25519-dalek` signing and verification, hardcoded QEMU keypairs) feature-gated under `ed25519`
- Injection-based signature testing: `AlternateSignatureScheme` + Byzantine cert forgery tests

### Validation coverage
- Stage 1 protocol coverage across proposer/prepare/commit/view-change/replay/persistence/dedup/malicious/validator-update/signature paths; feature-gated Ed25519 tests covering real sign/verify, cross-peer verification, tampered sig rejection, and forged-cert rejection
- Stage 2 cluster coverage across round progression/view-change/replay/persistence/malicious behavior/message validation/validator updates; Byzantine cert forgery cluster test
- Stage 3 embedded coverage (QEMU): 9-act scenario covering multi-round consensus, view change, client transaction submission, overdraft/nonce/gas-limit rejection, validator set update, WAL byte round-trip, and Ed25519 `PreparedCertificate` proof

---

## Missing Features

### Protocol/consensus capabilities (next major scope)
- Weighted voting / stake-aware quorum
- Pipelined consensus
- Signature aggregation (BLS path)
- Optimistic responsiveness fast path
- Proposal buffering / speculative execution
- Dynamic validator discovery
- Byzantine evidence collection and slashing pipeline

### Operational/productization
- Metrics and observability (structured metrics/logging dashboards)
- Production infrastructure swaps and hardening guidance (RocksDB/TCP/system timer/disk WAL)

### Platform parity
- Stage 3 embedded parity for WAL/restart recovery: WAL byte round-trip verified in QEMU (Act 8), but a full crash-recovery simulation (terminate process mid-consensus, restart, confirm state restored from WAL) is not yet implemented
- Malicious node injection and message deduplication observability in the embedded scenario
- Weighted voting, BLS aggregation, and optimistic fast path are not yet wired at Stage 3

---

## Current Status Summary

- Phase 1: complete
- Phase 2.1–2.7: complete
- Crypto abstraction with real Ed25519 signatures: complete
- Stage 3 scenario coverage (Acts 0–9): complete
- Remaining scope: Phase 3+ protocol features (weighted voting, pipelining, BLS, slashing) and Stage 3 parity (full crash-recovery restart, malicious injection in QEMU)
