---- MODULE MC_IBFTConsensus_CI ----
\* Honest-only reference model — no Byzantine validators.
\* Retained as a baseline to compare state space sizes against the Byzantine
\* quick check. Not used by any script directly.

EXTENDS IBFTConsensus

MC_Validators    == {0, 1, 2, 3}
MC_ByzValidators == {}
MC_Blocks        == {"b1", "b2"}
MC_MaxRound      == 1

====
