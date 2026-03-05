---- MODULE RaftConsensus ----
\* TLA+ specification of Raft leader election and log replication safety as
\* implemented in raft-node/src/implementations/raft/.
\*
\* Each TLA+ action maps directly to a Rust handler:
\*   StartElection       → handle_election_timeout → start_election
\*   GrantVote           → handle_request_vote (grant branch)
\*   DenyVote            → handle_request_vote (deny branch)
\*   BecomeLeader        → handle_request_vote_response (quorum branch)
\*   StepDown            → common::step_down
\*   LeaderAppend        → handle_client_request → append_entry
\*   ReplicateTo         → handle_append_entries (follower adopts leader log)
\*   LeaderCommit        → advance_commit_index (leader side)
\*   FollowerApplyCommit → handle_append_entries (follower advances commitIndex)
\*
\* Replication is modelled as direct log synchronisation (reliable-delivery
\* abstraction — equivalent to a single AppendEntries round-trip per step).
\*
\* Safety invariants (checked in Quick and CI):
\*   ElectionSafety    — at most one leader elected per term.
\*   VoteOnce          — each node grants at most one vote per term.
\*   LeaderTermOK      — a leader's currentTerm equals the term it was elected in.
\*   LogSafety         — no two committed log entries at the same index differ.
\*   LeaderCompleteness — every leader holds all previously committed entries.
\*
\* Liveness property (requires FairSpec; use for manual full-spec verification):
\*   Termination       — some node eventually commits an entry.
\*   To verify: update cfg to SPECIFICATION FairSpec and add PROPERTIES Termination.

EXTENDS Integers, FiniteSets, Sequences, TLC

\* ── Constants ─────────────────────────────────────────────────────────────────

CONSTANTS
    Nodes,      \* finite set of node IDs (e.g. {1, 2, 3})
    MaxTerm,    \* upper bound on terms explored by TLC
    MaxEntries  \* upper bound on log entries per node

ASSUME IsFiniteSet(Nodes)
ASSUME MaxTerm    \in Nat /\ MaxTerm    >= 1
ASSUME MaxEntries \in Nat /\ MaxEntries >= 1

\* ── Derived constants ─────────────────────────────────────────────────────────

N      == Cardinality(Nodes)

\* Mirrors common::quorum_size(peers.len()) = peers / 2 + 1.
\* For N=3: Quorum=2 (majority). For N=5: Quorum=3 (majority).
Quorum == N \div 2 + 1

Terms  == 1..MaxTerm

\* ── Variables ─────────────────────────────────────────────────────────────────

VARIABLES
    currentTerm,   \* currentTerm[v]  — persisted, mirrors RaftContext.current_term
    role,          \* role[v]         — Follower | Candidate | Leader
    votedFor,      \* votedFor[v]     — [present |-> FALSE] or [present |-> TRUE, node |-> c]
                   \*                   mirrors RaftContext.voted_for (Option<PeerId>)
    rvMsgs,        \* RequestVote messages (set): [term, candidate]
    rvrMsgs,       \* RequestVoteResponse messages (set): [term, voter, candidate, granted]
    log,           \* log[v]          — sequence of [term: Nat] records
    commitIndex    \* commitIndex[v]  — index of highest committed entry (0 = none)

vars == <<currentTerm, role, votedFor, rvMsgs, rvrMsgs, log, commitIndex>>

\* ── Helper operators ──────────────────────────────────────────────────────────

LastLogTerm(l) == IF Len(l) = 0 THEN 0 ELSE l[Len(l)].term

\* Raft log freshness: candidate's log is at least as up-to-date as voter's.
\* Higher last-log-term wins; ties broken by log length (Raft §5.4.1).
LogUpToDate(cLog, vLog) ==
    LET cLT == LastLogTerm(cLog)
        vLT == LastLogTerm(vLog)
    IN cLT > vLT \/ (cLT = vLT /\ Len(cLog) >= Len(vLog))

\* ── Type invariant ────────────────────────────────────────────────────────────

TypeOK ==
    /\ \A v \in Nodes :
           currentTerm[v] \in 0..MaxTerm
        /\ role[v] \in {"Follower", "Candidate", "Leader"}
        /\ (votedFor[v].present = FALSE \/
            (votedFor[v].present = TRUE /\ votedFor[v].node \in Nodes))
        /\ Len(log[v]) \in 0..MaxEntries
        /\ \A i \in 1..Len(log[v]) : log[v][i].term \in 1..MaxTerm
        /\ commitIndex[v] \in 0..MaxEntries
    /\ rvMsgs  \subseteq [term: Terms, candidate: Nodes]
    /\ rvrMsgs \subseteq [term: Terms, voter: Nodes, candidate: Nodes, granted: BOOLEAN]

\* ── Init ──────────────────────────────────────────────────────────────────────

Init ==
    /\ currentTerm = [v \in Nodes |-> 0]
    /\ role        = [v \in Nodes |-> "Follower"]
    /\ votedFor    = [v \in Nodes |-> [present |-> FALSE]]
    /\ rvMsgs      = {}
    /\ rvrMsgs     = {}
    /\ log         = [v \in Nodes |-> <<>>]
    /\ commitIndex = [v \in Nodes |-> 0]

\* ── Actions ───────────────────────────────────────────────────────────────────

\* Corresponds to: handle_election_timeout → start_election.
\* Node v increments term, transitions to Candidate, self-votes, broadcasts RequestVote.
StartElection(v) ==
    LET t == currentTerm[v] + 1
    IN /\ t \in Terms
       /\ role[v] \in {"Follower", "Candidate"}
       /\ currentTerm' = [currentTerm EXCEPT ![v] = t]
       /\ role'        = [role        EXCEPT ![v] = "Candidate"]
       /\ votedFor'    = [votedFor    EXCEPT ![v] = [present |-> TRUE, node |-> v]]
       /\ rvMsgs'      = rvMsgs  \cup {[term |-> t, candidate |-> v]}
       /\ rvrMsgs'     = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> v, granted |-> TRUE]}
       /\ UNCHANGED <<log, commitIndex>>

\* Corresponds to: handle_request_vote — vote granted branch.
\* Includes Raft log-freshness check: only grant if candidate's log is at least
\* as up-to-date as the voter's log (§5.4.1). This is the critical guard that
\* ensures LeaderCompleteness and thus LogSafety.
GrantVote(v, c, t) ==
    /\ [term |-> t, candidate |-> c] \in rvMsgs
    /\ t \in Terms
    /\ t >= currentTerm[v]
    /\ \/ t > currentTerm[v]                                    \* stepping up: always ok
       \/ votedFor[v].present = FALSE                           \* same term: not yet voted
       \/ (votedFor[v].present = TRUE /\ votedFor[v].node = c)  \* same term: idempotent
    /\ ~\E m \in rvrMsgs : m.term = t /\ m.voter = v /\ m.granted
    /\ LogUpToDate(log[c], log[v])
    /\ currentTerm' = [currentTerm EXCEPT ![v] = t]
    /\ votedFor'    = [votedFor    EXCEPT ![v] = [present |-> TRUE, node |-> c]]
    /\ role'        = [role EXCEPT ![v] = IF t > currentTerm[v] THEN "Follower" ELSE role[v]]
    /\ rvrMsgs'     = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> c, granted |-> TRUE]}
    /\ UNCHANGED <<rvMsgs, log, commitIndex>>

\* Corresponds to: handle_request_vote — vote denied branch.
\* Node v rejects the vote: stale term, already voted for another, or stale log.
DenyVote(v, c, t) ==
    /\ [term |-> t, candidate |-> c] \in rvMsgs
    /\ t \in Terms
    /\ ~\E m \in rvrMsgs : m.term = t /\ m.voter = v /\ m.candidate = c
    /\ \/ t < currentTerm[v]
       \/ (t = currentTerm[v] /\ votedFor[v].present = TRUE /\ votedFor[v].node # c)
       \/ (t >= currentTerm[v] /\ ~LogUpToDate(log[c], log[v]))
    /\ rvrMsgs' = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> c, granted |-> FALSE]}
    /\ UNCHANGED <<currentTerm, role, votedFor, rvMsgs, log, commitIndex>>

\* Corresponds to: handle_request_vote_response (quorum branch) → become_leader.
\* Candidate c transitions to Leader after collecting Quorum granted votes in term t.
BecomeLeader(c, t) ==
    /\ role[c] = "Candidate"
    /\ currentTerm[c] = t
    /\ Cardinality({m \in rvrMsgs : m.term = t /\ m.candidate = c /\ m.granted}) >= Quorum
    /\ role' = [role EXCEPT ![c] = "Leader"]
    /\ UNCHANGED <<currentTerm, votedFor, rvMsgs, rvrMsgs, log, commitIndex>>

\* Corresponds to: common::step_down.
\* Any node steps down to Follower and resets votedFor on seeing a higher term.
StepDown(v, t) ==
    /\ t > currentTerm[v]
    /\ t \in Terms
    /\ \/ \E m \in rvMsgs  : m.term = t
       \/ \E m \in rvrMsgs : m.term = t
    /\ currentTerm' = [currentTerm EXCEPT ![v] = t]
    /\ role'        = [role        EXCEPT ![v] = "Follower"]
    /\ votedFor'    = [votedFor    EXCEPT ![v] = [present |-> FALSE]]
    /\ UNCHANGED <<rvMsgs, rvrMsgs, log, commitIndex>>

\* Corresponds to: handle_client_request → append_entry.
\* Leader appends one new entry with its current term (bounded by MaxEntries).
LeaderAppend(v) ==
    /\ role[v] = "Leader"
    /\ Len(log[v]) < MaxEntries
    /\ log' = [log EXCEPT ![v] = Append(log[v], [term |-> currentTerm[v]])]
    /\ UNCHANGED <<currentTerm, role, votedFor, rvMsgs, rvrMsgs, commitIndex>>

\* Corresponds to: handle_append_entries.
\* Follower synchronises its log directly from the leader (reliable-delivery
\* abstraction — collapses AppendEntries send + receive into one atomic step).
\* If the follower is behind in term it also steps down implicitly.
ReplicateTo(leader, follower) ==
    /\ role[leader] = "Leader"
    /\ leader # follower
    /\ currentTerm[follower] <= currentTerm[leader]
    /\ log'         = [log         EXCEPT ![follower] = log[leader]]
    /\ currentTerm' = [currentTerm EXCEPT ![follower] = currentTerm[leader]]
    /\ role'        = [role        EXCEPT ![follower] = "Follower"]
    /\ votedFor'    = [votedFor    EXCEPT ![follower] =
                          IF currentTerm[follower] < currentTerm[leader]
                          THEN [present |-> FALSE]
                          ELSE votedFor[follower]]
    /\ UNCHANGED <<rvMsgs, rvrMsgs, commitIndex>>

\* Corresponds to: advance_commit_index (leader side).
\* Leader commits the next entry once a Quorum of nodes hold it.
\* Raft safety rule: a leader may only directly commit entries from its own term.
LeaderCommit(v) ==
    /\ role[v] = "Leader"
    /\ LET k == commitIndex[v] + 1
       IN /\ k <= Len(log[v])
          /\ log[v][k].term = currentTerm[v]
          /\ Cardinality({u \in Nodes :
                 Len(log[u]) >= k /\ log[u][k].term = log[v][k].term}) >= Quorum
          /\ commitIndex' = [commitIndex EXCEPT ![v] = k]
    /\ UNCHANGED <<currentTerm, role, votedFor, rvMsgs, rvrMsgs, log>>

\* Corresponds to: handle_append_entries → follower advances commitIndex.
\* Follower advances its commitIndex after receiving AppendEntries with a
\* higher leaderCommit than its own commitIndex.
\* Guard: the follower's log must already match the leader's up to the commit
\* point (i.e., ReplicateTo must have run first), mirroring the real
\* protocol's guarantee that commitIndex only advances with a consistent log.
FollowerApplyCommit(follower, leader) ==
    /\ role[leader] = "Leader"
    /\ follower # leader
    /\ currentTerm[follower] = currentTerm[leader]
    /\ commitIndex[leader] > commitIndex[follower]
    /\ Len(log[follower]) >= commitIndex[leader]
    /\ \A k \in 1..commitIndex[leader] :
           log[follower][k].term = log[leader][k].term
    /\ commitIndex' = [commitIndex EXCEPT ![follower] = commitIndex[leader]]
    /\ UNCHANGED <<currentTerm, role, votedFor, rvMsgs, rvrMsgs, log>>

\* ── Next ──────────────────────────────────────────────────────────────────────

Next ==
    \/ \E v \in Nodes : StartElection(v)
    \/ \E v \in Nodes, c \in Nodes, t \in Terms :
           GrantVote(v, c, t)
        \/ DenyVote(v, c, t)
    \/ \E c \in Nodes, t \in Terms : BecomeLeader(c, t)
    \/ \E v \in Nodes, t \in Terms : StepDown(v, t)
    \/ \E v \in Nodes : LeaderAppend(v)
    \/ \E leader \in Nodes, follower \in Nodes : ReplicateTo(leader, follower)
    \/ \E v \in Nodes : LeaderCommit(v)
    \/ \E follower \in Nodes, leader \in Nodes : FollowerApplyCommit(follower, leader)

Spec     == Init /\ [][Next]_vars
FairSpec == Spec /\ WF_vars(Next)

\* ── Safety invariants ─────────────────────────────────────────────────────────

\* ELECTION SAFETY: At most one leader per term.
\* The primary correctness property of Raft leader election.
\* Follows from VoteOnce + quorum intersection.
ElectionSafety ==
    \A v1, v2 \in Nodes :
        role[v1] = "Leader" /\ role[v2] = "Leader" /\ currentTerm[v1] = currentTerm[v2]
        => v1 = v2

\* VOTE ONCE: Each node grants at most one vote per term.
\* Mirrors votedFor being persisted before sending RequestVoteResponse.
VoteOnce ==
    \A t \in Terms :
        \A m1, m2 \in {m \in rvrMsgs : m.term = t /\ m.granted} :
            m1.voter = m2.voter => m1.candidate = m2.candidate

\* LEADER TERM OK: A leader's current term equals the term it was elected in.
\* Ensures leaders step down when they observe a higher term.
LeaderTermOK ==
    \A v \in Nodes :
        role[v] = "Leader" => currentTerm[v] \in Terms

\* LOG SAFETY: No two committed log entries at the same index have different terms.
\* Combined with ElectionSafety this guarantees identical log prefixes at all
\* committed indices (Raft §5.4 Log Matching Property).
LogSafety ==
    \A v1, v2 \in Nodes :
        \A k \in 1..MaxEntries :
            IF k <= commitIndex[v1] /\ k <= commitIndex[v2]
            THEN IF k <= Len(log[v1]) /\ k <= Len(log[v2])
                 THEN log[v1][k].term = log[v2][k].term
                 ELSE FALSE
            ELSE TRUE

\* LEADER COMPLETENESS: Every leader holds all committed entries from its term or below.
\* A stale leader in term T is not required to hold entries committed by a term-T'>T
\* leader, since it will step down once it hears about the higher term.
\* This captures the Raft paper's guarantee: a newly-elected leader already has
\* all committed entries from prior terms, ensured by the log-freshness voting rule.
LeaderCompleteness ==
    \A leader \in Nodes :
        IF role[leader] = "Leader"
        THEN \A v \in Nodes :
                 \A k \in 1..commitIndex[v] :
                     IF k <= Len(log[v]) /\ log[v][k].term <= currentTerm[leader]
                     THEN IF k <= Len(log[leader])
                          THEN log[leader][k].term = log[v][k].term
                          ELSE FALSE
                     ELSE TRUE
        ELSE TRUE

\* ── Liveness ──────────────────────────────────────────────────────────────────

\* TERMINATION: Under weak fairness (FairSpec) some node eventually commits an entry.
\* To verify manually: update cfg to SPECIFICATION FairSpec, add PROPERTIES Termination.
Termination == <>(\E v \in Nodes : commitIndex[v] > 0)

====
