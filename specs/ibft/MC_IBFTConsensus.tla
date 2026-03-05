---- MODULE MC_IBFTConsensus ----
\* Full model-checking overrides for IBFTConsensus.
\*
\* N=4  F=1  Quorum=3  ByzValidators={3}  MaxRound=2
\* Includes one Byzantine validator and explores two full view-change cycles.
\* State space is large — complete BFS requires 30+ minutes.
\* Run via: scripts\ibft_run_tla_full.ps1

EXTENDS IBFTConsensus

MC_Validators    == {0, 1, 2, 3}
MC_ByzValidators == {3}
MC_Blocks        == {"b1", "b2"}

\* Explore up to round 2: one honest proposal + two view-change cycles.
MC_MaxRound == 2

====
