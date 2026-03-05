---- MODULE MC_IBFTConsensus_Quick ----
\* Byzantine-focused quick model check for IBFTConsensus.
\*
\* Uses MaxRound=0 (round 0 only, no view changes) so TLC can exhaustively
\* explore the Byzantine commit-phase attack in seconds.
\*
\* What this proves under F=1 Byzantine injection of Prepare/Commit:
\*   Agreement                  — correct validators cannot decide different blocks.
\*   CommitImpliesPrepareQuorum — correct commits only follow quorum Prepare.
\*   LockConsistency            — locked values always have a quorum certificate.
\*   QuorumIntersection         — any two quorums of size 3 share a member.
\*
\* The Byzantine validator (3) may inject Prepare("b2") and Commit("b2") freely,
\* but since PrepareCount("b2") can reach at most 1 (one Byzantine sender),
\* it cannot reach the quorum of 3 needed to flip a correct validator's commit.
\* TLC exhaustively verifies this.
\*
\* View-change safety (locked-block re-propose) is covered by the honest model
\* run (MC_IBFTConsensus_CI, MaxRound=1) also invoked in ibft_run_tla_quick.ps1.
\*
\* N=4  F=1  Quorum=3  ByzValidators={3}  MaxRound=0
\* Target: complete BFS in under 5 seconds.

EXTENDS IBFTConsensus

MC_Validators    == {0, 1, 2, 3}
MC_ByzValidators == {3}
MC_Blocks        == {"b1", "b2"}
MC_MaxRound      == 0

====
