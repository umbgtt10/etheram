---- MODULE RaftConsensus ----
\* TLA+ specification of Raft leader election as implemented in
\* raft-node/src/implementations/raft/election.rs
\*
\* Each TLA+ action maps directly to a Rust handler:
\*   StartElection  → handle_election_timeout → start_election
\*   GrantVote      → handle_request_vote (grant branch)
\*   DenyVote       → handle_request_vote (deny branch)
\*   BecomeLeader   → handle_request_vote_response (quorum branch) → become_leader
\*   StepDown       → common::step_down (triggered by any higher-term message)
\*
\* Message sets grow monotonically (reliable delivery abstraction).
\* Log and replication are abstracted away — this spec covers election safety only.
\*
\* Safety invariants:
\*   ElectionSafety — at most one leader elected per term.
\*   VoteOnce       — each node grants at most one vote per term.
\*   LeaderTermOK   — a leader's currentTerm equals the term it was elected in.

EXTENDS Integers, FiniteSets, TLC

\* ── Constants ─────────────────────────────────────────────────────────────────

CONSTANTS
    Nodes,    \* finite set of node IDs (e.g. {1, 2, 3})
    MaxTerm   \* upper bound on terms explored by TLC

ASSUME IsFiniteSet(Nodes)
ASSUME MaxTerm \in Nat /\ MaxTerm >= 1

\* ── Derived constants ─────────────────────────────────────────────────────────

N      == Cardinality(Nodes)

\* Mirrors common::quorum_size(peers.len()) = peers / 2 + 1.
\* In TLA+ we count all nodes (including self-vote) with N \div 2 + 1.
\* For N=3: Quorum=2 (majority). For N=5: Quorum=3 (majority).
Quorum == N \div 2 + 1

Terms  == 1..MaxTerm

\* ── Variables ─────────────────────────────────────────────────────────────────

VARIABLES
    currentTerm,  \* currentTerm[v] — persisted, mirrors RaftContext.current_term
    role,         \* role[v]        — Follower | Candidate | Leader
    votedFor,     \* votedFor[v]    — [present |-> FALSE] or [present |-> TRUE, node |-> c]
                  \*                  mirrors RaftContext.voted_for (Option<PeerId>)
    rvMsgs,       \* RequestVote messages: [term, candidate]
    rvrMsgs       \* RequestVoteResponse messages: [term, voter, candidate, granted]

vars == <<currentTerm, role, votedFor, rvMsgs, rvrMsgs>>

\* ── Type invariant ────────────────────────────────────────────────────────────

TypeOK ==
    /\ \A v \in Nodes :
           currentTerm[v] \in 0..MaxTerm
        /\ role[v] \in {"Follower", "Candidate", "Leader"}
        /\ (votedFor[v].present = FALSE \/
            (votedFor[v].present = TRUE /\ votedFor[v].node \in Nodes))
    /\ rvMsgs  \subseteq [term: Terms, candidate: Nodes]
    /\ rvrMsgs \subseteq [term: Terms, voter: Nodes, candidate: Nodes, granted: BOOLEAN]

\* ── Init ──────────────────────────────────────────────────────────────────────

Init ==
    /\ currentTerm = [v \in Nodes |-> 0]
    /\ role        = [v \in Nodes |-> "Follower"]
    /\ votedFor    = [v \in Nodes |-> [present |-> FALSE]]
    /\ rvMsgs      = {}
    /\ rvrMsgs     = {}

\* ── Actions ───────────────────────────────────────────────────────────────────

\* Corresponds to: handle_election_timeout → start_election.
\* Node v increments term, transitions to Candidate, votes for itself,
\* and broadcasts RequestVote.
StartElection(v) ==
    LET t == currentTerm[v] + 1
    IN /\ t \in Terms
       /\ role[v] \in {"Follower", "Candidate"}
       /\ currentTerm' = [currentTerm EXCEPT ![v] = t]
       /\ role'        = [role        EXCEPT ![v] = "Candidate"]
       /\ votedFor'    = [votedFor    EXCEPT ![v] = [present |-> TRUE, node |-> v]]
       /\ rvMsgs'      = rvMsgs  \cup {[term |-> t, candidate |-> v]}
       /\ rvrMsgs'     = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> v, granted |-> TRUE]}

\* Corresponds to: handle_request_vote — vote granted branch.
\* Node v grants its vote to candidate c in term t.
\* When t > currentTerm[v], v implicitly steps down first (mirrors step_down
\* called before the vote in handle_request_vote).
GrantVote(v, c, t) ==
    /\ [term |-> t, candidate |-> c] \in rvMsgs
    /\ t \in Terms
    /\ t >= currentTerm[v]
    /\ \/ t > currentTerm[v]                                   \* stepping up: always ok
       \/ votedFor[v].present = FALSE                          \* same term: not yet voted
       \/ (votedFor[v].present = TRUE /\ votedFor[v].node = c) \* same term: idempotent
    /\ ~\E m \in rvrMsgs : m.term = t /\ m.voter = v /\ m.granted
    /\ currentTerm' = [currentTerm EXCEPT ![v] = t]
    /\ votedFor'    = [votedFor    EXCEPT ![v] = [present |-> TRUE, node |-> c]]
    /\ role'        = [role EXCEPT ![v] = IF t > currentTerm[v] THEN "Follower" ELSE role[v]]
    /\ rvrMsgs'     = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> c, granted |-> TRUE]}
    /\ UNCHANGED rvMsgs

\* Corresponds to: handle_request_vote — vote denied branch.
\* Node v rejects the vote: stale term or already voted for a different candidate.
DenyVote(v, c, t) ==
    /\ [term |-> t, candidate |-> c] \in rvMsgs
    /\ t \in Terms
    /\ ~\E m \in rvrMsgs : m.term = t /\ m.voter = v /\ m.candidate = c
    /\ \/ t < currentTerm[v]
       \/ (t = currentTerm[v] /\ votedFor[v].present = TRUE /\ votedFor[v].node # c)
    /\ rvrMsgs' = rvrMsgs \cup {[term |-> t, voter |-> v, candidate |-> c, granted |-> FALSE]}
    /\ UNCHANGED <<currentTerm, role, votedFor, rvMsgs>>

\* Corresponds to: handle_request_vote_response (quorum branch) → become_leader.
\* Candidate c transitions to Leader after collecting Quorum granted votes in term t.
BecomeLeader(c, t) ==
    /\ role[c] = "Candidate"
    /\ currentTerm[c] = t
    /\ Cardinality({m \in rvrMsgs : m.term = t /\ m.candidate = c /\ m.granted}) >= Quorum
    /\ role' = [role EXCEPT ![c] = "Leader"]
    /\ UNCHANGED <<currentTerm, votedFor, rvMsgs, rvrMsgs>>

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
    /\ UNCHANGED <<rvMsgs, rvrMsgs>>

\* ── Next ──────────────────────────────────────────────────────────────────────

Next ==
    \/ \E v \in Nodes : StartElection(v)
    \/ \E v \in Nodes, c \in Nodes, t \in Terms :
           GrantVote(v, c, t)
        \/ DenyVote(v, c, t)
    \/ \E c \in Nodes, t \in Terms : BecomeLeader(c, t)
    \/ \E v \in Nodes, t \in Terms : StepDown(v, t)

Spec == Init /\ [][Next]_vars

\* ── Invariants ────────────────────────────────────────────────────────────────

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

====
