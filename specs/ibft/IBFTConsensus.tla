---- MODULE IBFTConsensus ----
\* TLA+ specification of the IBFT consensus protocol as implemented in
\* etheram-node/src/implementations/ibft/ibft_protocol.rs
\*
\* Models a single consensus height. The state machine mirrors handle_message()
\* directly: each TLA+ action corresponds to one Rust message handler.
\*
\* Safety properties checked:
\*   Agreement                 — no two correct validators decide different blocks.
\*   CommitImpliesPrepareQuorum — correct validators only commit after quorum prepared.
\*   LockConsistency           — locked block always has a quorum prepare certificate.
\*   QuorumIntersection        — mathematical basis: any two quorums share a validator.
\*
\* Byzantine model:
\*   ByzValidators is a subset of Validators that may inject arbitrary messages
\*   (PrePrepare, Prepare, Commit) without following the correct protocol guards.
\*   Pass ByzValidators = {} for the pure honest model.
\*   Pass ByzValidators = {k} for a single faulty validator scenario (F=1).

EXTENDS Integers, FiniteSets, TLC

\* ── Constants (instantiated in MC_IBFTConsensus*.tla) ─────────────────────────

CONSTANTS
    Validators,    \* finite set of validator IDs (e.g. {0,1,2,3})
    ByzValidators, \* subset of Validators that behave Byzantine (may be {})
    Blocks,        \* finite set of block values — abstracts away hash content
    MaxRound       \* upper bound on rounds explored by TLC

ASSUME IsFiniteSet(Validators)
ASSUME ByzValidators \subseteq Validators
ASSUME MaxRound \in Nat

\* Correct validators are those not in the Byzantine set.
CorrectValidators == Validators \ ByzValidators

\* ── Derived constants ─────────────────────────────────────────────────────────

N       == Cardinality(Validators)
F       == (N - 1) \div 3

\* ValidatorSet::quorum_size() = 2 * n / 3 + 1  (integer division)
Quorum  == (2 * N) \div 3 + 1

Rounds  == 0..MaxRound

\* Proposer(r) mirrors ValidatorSet::get_proposer_for_round(height=0, round=r).
\* Validators must be 0..N-1 for this formula to hold.
\* Corresponds to: validators[(0 + round) % validators.len()]
Proposer(r) == r % N

\* ── Variables ─────────────────────────────────────────────────────────────────
\*
\* Message sets grow monotonically — once broadcast, a message is always
\* available to every correct validator. This abstracts reliable delivery.
\*
\* Per-node state mirrors IbftProtocol struct fields.

VARIABLES
    \* --- message sets (global) ---
    ppMsgs,      \* PrePrepare:  [round, block, sender]
    prepMsgs,    \* Prepare:     [round, block, sender]
    commitMsgs,  \* Commit:      [round, block, sender]
    vcMsgs,      \* ViewChange:  [round, sender, cert]   cert = "none" | [round, block]
    nvMsgs,      \* NewView:     [round, block, sender]

    \* --- per-node state ---
    nround,      \* nround[v]      = current round (integer)
    decided,     \* decided[v]     = decided block value or "none"
    locked,      \* locked[v]      = [round, block] or "none"  (PreparedCertificate)
    prepSent,    \* prepSent[v][r] = TRUE after v broadcasts Prepare in round r
    commitSent   \* commitSent[v][r] = TRUE after v broadcasts Commit in round r

vars == <<ppMsgs, prepMsgs, commitMsgs, vcMsgs, nvMsgs,
          nround, decided, locked, prepSent, commitSent>>

\* ── Type invariant ────────────────────────────────────────────────────────────

TypeOK ==
    /\ ppMsgs     \subseteq [round: Rounds, block: Blocks, sender: Validators]
    /\ prepMsgs   \subseteq [round: Rounds, block: Blocks, sender: Validators]
    /\ commitMsgs \subseteq [round: Rounds, block: Blocks, sender: Validators]
    /\ \A v \in Validators :
           nround[v] \in Rounds
        /\ decided[v]  \in Blocks \cup {"none"}
        /\ IF locked[v].present = FALSE THEN TRUE
           ELSE locked[v].round \in Rounds /\ locked[v].block \in Blocks

\* ── Helper operators ──────────────────────────────────────────────────────────

\* Number of distinct validators who sent Prepare for (round r, block b).
\* Mirrors VoteTracker::has_quorum logic.
PrepareCount(r, b) ==
    Cardinality({m \in prepMsgs : m.round = r /\ m.block = b})

CommitCount(r, b) ==
    Cardinality({m \in commitMsgs : m.round = r /\ m.block = b})

ViewChangeCount(r) ==
    Cardinality({m \in vcMsgs : m.round = r})

\* The highest-round PreparedCertificate carried by ViewChange messages for
\* round r. Mirrors the highest-round-cert-wins rule in handle_view_change().
BestCertForRound(r) ==
    LET certs == {m.cert : m \in {vc \in vcMsgs : vc.round = r /\ vc.cert.present = TRUE}}
    IN IF certs = {}
       THEN [present |-> FALSE]
       ELSE CHOOSE c \in certs : \A c2 \in certs : c.round >= c2.round

\* ── Init ──────────────────────────────────────────────────────────────────────

Init ==
    /\ ppMsgs     = {}
    /\ prepMsgs   = {}
    /\ commitMsgs = {}
    /\ vcMsgs     = {}
    /\ nvMsgs     = {}
    /\ nround     = [v \in Validators |-> 0]
    /\ decided    = [v \in Validators |-> "none"]
    /\ locked     = [v \in Validators |-> [present |-> FALSE]]
    /\ prepSent   = [v \in Validators |-> [r \in Rounds |-> FALSE]]
    /\ commitSent = [v \in Validators |-> [r \in Rounds |-> FALSE]]

\* ── Actions ───────────────────────────────────────────────────────────────────

\* Corresponds to: handle_timer_propose_block / handle_pre_prepare (proposer side).
\* The proposer for round r broadcasts a PrePrepare.
\* If locked: must re-propose the locked block (locked-block re-propose invariant).
\* If not locked: free to propose any block in Blocks.
Propose(v, r, b) ==
    /\ Proposer(r) = v
    /\ nround[v] = r
    /\ ~\E m \in ppMsgs : m.round = r        \* one PrePrepare per round
    /\ IF locked[v].present = FALSE THEN TRUE ELSE locked[v].block = b
    /\ ppMsgs' = ppMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ UNCHANGED <<prepMsgs, commitMsgs, vcMsgs, nvMsgs,
                   nround, decided, locked, prepSent, commitSent>>

\* Corresponds to: handle_pre_prepare (non-proposer side).
\* Validator v receives a PrePrepare and broadcasts Prepare.
\* Guard: valid PrePrepare exists FROM THE ROUND PROPOSER, v hasn't prepared this
\*        round, and the proposed block is compatible with v's lock.
\* The sender check mirrors handle_pre_prepare's is_proposer_for_round guard.
SendPrepare(v, r, b) ==
    /\ ~prepSent[v][r]
    /\ nround[v] = r
    /\ \E m \in ppMsgs : m.round = r /\ m.block = b /\ m.sender = Proposer(r)
    /\ IF locked[v].present = FALSE THEN TRUE ELSE locked[v].block = b
    /\ prepMsgs' = prepMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ prepSent' = [prepSent EXCEPT ![v][r] = TRUE]
    /\ UNCHANGED <<ppMsgs, commitMsgs, vcMsgs, nvMsgs,
                   nround, decided, locked, commitSent>>

\* Corresponds to: handle_prepare — quorum branch.
\* When quorum Prepares exist, v locks and broadcasts Commit.
\* Mirrors: prepared_certificate set, commit_sent = true.
SendCommit(v, r, b) ==
    /\ ~commitSent[v][r]
    /\ nround[v] = r
    /\ PrepareCount(r, b) >= Quorum
    /\ commitMsgs' = commitMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ commitSent' = [commitSent EXCEPT ![v][r] = TRUE]
    /\ locked'     = [locked EXCEPT ![v] = [present |-> TRUE, round |-> r, block |-> b]]
    /\ UNCHANGED <<ppMsgs, prepMsgs, vcMsgs, nvMsgs,
                   nround, decided, prepSent>>

\* Corresponds to: handle_commit — quorum branch (StoreBlock + IncrementHeight).
\* Validator v decides once it sees quorum Commits.
Decide(v, r, b) ==
    /\ decided[v] = "none"
    /\ CommitCount(r, b) >= Quorum
    /\ decided' = [decided EXCEPT ![v] = b]
    /\ UNCHANGED <<ppMsgs, prepMsgs, commitMsgs, vcMsgs, nvMsgs,
                   nround, locked, prepSent, commitSent>>

\* Corresponds to: handle_timer_timeout_round.
\* Validator v advances to round r and broadcasts ViewChange carrying its lock.
SendViewChange(v, r) ==
    /\ r \in Rounds
    /\ nround[v] < r
    /\ decided[v] = "none"
    /\ ~\E m \in vcMsgs : m.round = r /\ m.sender = v
    /\ nround' = [nround EXCEPT ![v] = r]
    /\ vcMsgs' = vcMsgs \cup {[round |-> r, sender |-> v, cert |-> locked[v]]}
    /\ UNCHANGED <<ppMsgs, prepMsgs, commitMsgs, nvMsgs,
                   decided, locked, prepSent, commitSent>>

\* Corresponds to: handle_view_change — quorum branch (proposer broadcasts NewView).
\* The proposer for round r collects quorum ViewChanges then sends NewView.
\* Block b is the re-proposal: the locked block from the highest-round cert,
\* or any block if no cert exists. Mirrors highest-round-cert-wins rule.
SendNewView(v, r, b) ==
    /\ Proposer(r) = v
    /\ nround[v] = r
    /\ ~\E m \in nvMsgs : m.round = r
    /\ ViewChangeCount(r) >= Quorum
    /\ LET best == BestCertForRound(r)
       IN IF best.present = FALSE THEN TRUE ELSE best.block = b
    /\ nvMsgs' = nvMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ UNCHANGED <<ppMsgs, prepMsgs, commitMsgs, vcMsgs,
                   nround, decided, locked, prepSent, commitSent>>

\* Corresponds to: handle_new_view.
\* Non-proposer v processes a NewView for round r and advances its round.
\* Mirrors: current_round = round, reset_for_new_round().
ProcessNewView(v, r) ==
    /\ nround[v] < r
    /\ \E m \in nvMsgs : m.round = r
    /\ nround' = [nround EXCEPT ![v] = r]
    /\ UNCHANGED <<ppMsgs, prepMsgs, commitMsgs, vcMsgs, nvMsgs,
                   decided, locked, prepSent, commitSent>>

\* ── Byzantine actions ─────────────────────────────────────────────────────────
\*
\* These actions model a Byzantine validator who may inject arbitrary messages
\* into the shared message sets without following any protocol guard.
\*
\* ByzInjectPrePrepare is only meaningful when the Byzantine validator is the
\* round proposer (Proposer(r) \in ByzValidators); otherwise SendPrepare's
\* sender = Proposer(r) guard ensures correct validators ignore it.
\*
\* Each action is gated on v \in ByzValidators and is a no-op when
\* ByzValidators = {}.

ByzInjectPrePrepare(v, r, b) ==
    /\ v \in ByzValidators
    /\ r \in Rounds
    /\ b \in Blocks
    /\ ppMsgs' = ppMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ UNCHANGED <<prepMsgs, commitMsgs, vcMsgs, nvMsgs,
                   nround, decided, locked, prepSent, commitSent>>

ByzInjectPrepare(v, r, b) ==
    /\ v \in ByzValidators
    /\ r \in Rounds
    /\ b \in Blocks
    /\ prepMsgs' = prepMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ UNCHANGED <<ppMsgs, commitMsgs, vcMsgs, nvMsgs,
                   nround, decided, locked, prepSent, commitSent>>

ByzInjectCommit(v, r, b) ==
    /\ v \in ByzValidators
    /\ r \in Rounds
    /\ b \in Blocks
    /\ commitMsgs' = commitMsgs \cup {[round |-> r, block |-> b, sender |-> v]}
    /\ UNCHANGED <<ppMsgs, prepMsgs, vcMsgs, nvMsgs,
                   nround, decided, locked, prepSent, commitSent>>

\* ── Next ──────────────────────────────────────────────────────────────────────

Next ==
    \/ \E v \in Validators, r \in Rounds, b \in Blocks :
           Propose(v, r, b)
        \/ SendPrepare(v, r, b)
        \/ SendCommit(v, r, b)
        \/ Decide(v, r, b)
        \/ SendNewView(v, r, b)
    \/ \E v \in Validators, r \in Rounds :
           SendViewChange(v, r)
        \/ ProcessNewView(v, r)
    \/ \E v \in ByzValidators, r \in Rounds, b \in Blocks :
           ByzInjectPrePrepare(v, r, b)
        \/ ByzInjectPrepare(v, r, b)
        \/ ByzInjectCommit(v, r, b)

Spec == Init /\ [][Next]_vars

\* ── Invariants ────────────────────────────────────────────────────────────────

\* AGREEMENT (safety): No two correct validators ever decide different blocks.
\* Byzantine validators are excluded — they may decide anything consistent with
\* the quorum guard in Decide, but we only assert safety for correct nodes.
Agreement ==
    \A v1, v2 \in CorrectValidators :
        decided[v1] # "none" /\ decided[v2] # "none"
        => decided[v1] = decided[v2]

\* COMMIT IMPLIES PREPARE QUORUM: A correct validator only commits after quorum
\* prepared. Byzantine validators may inject Commit without this guard, so the
\* invariant is stated only for correct validator commit messages.
CommitImpliesPrepareQuorum ==
    \A r \in Rounds, b \in Blocks :
        Cardinality({m \in commitMsgs : m.round = r /\ m.block = b
                                        /\ m.sender \in CorrectValidators}) > 0
        => PrepareCount(r, b) >= Quorum

\* LOCK CONSISTENCY: A node's locked block is always one for which quorum
\* prepared. Mirrors: prepared_certificate only set inside handle_prepare
\* after has_quorum check.
LockConsistency ==
    \A v \in Validators :
        locked[v].present = TRUE
        => PrepareCount(locked[v].round, locked[v].block) >= Quorum

\* QUORUM INTERSECTION: Any two quorums share at least one validator.
\* This is the mathematical foundation for Agreement — it implies two quorum
\* prepare sets cannot be for different blocks at the same round.
QuorumIntersection ==
    \A S1, S2 \in SUBSET Validators :
        Cardinality(S1) >= Quorum /\ Cardinality(S2) >= Quorum
        => S1 \cap S2 # {}

\* ── Liveness property ─────────────────────────────────────────────────────────

\* TERMINATION: Under weak fairness on all actions, all correct validators
\* eventually decide. Only meaningful with concrete constants and fairness.
Termination ==
    <>(\A v \in CorrectValidators : decided[v] # "none")

====
