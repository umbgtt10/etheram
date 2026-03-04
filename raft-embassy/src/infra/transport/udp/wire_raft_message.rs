// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::snapshot::RaftSnapshot;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireLogEntry {
    pub index: u64,
    pub term: u64,
    pub payload: Vec<u8>,
}

impl From<LogEntry<Vec<u8>>> for WireLogEntry {
    fn from(e: LogEntry<Vec<u8>>) -> Self {
        Self {
            index: e.index,
            term: e.term,
            payload: e.payload,
        }
    }
}

impl From<WireLogEntry> for LogEntry<Vec<u8>> {
    fn from(w: WireLogEntry) -> Self {
        Self {
            index: w.index,
            term: w.term,
            payload: w.payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireRaftSnapshot {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub data: Vec<u8>,
}

impl From<RaftSnapshot> for WireRaftSnapshot {
    fn from(s: RaftSnapshot) -> Self {
        Self {
            last_included_index: s.last_included_index,
            last_included_term: s.last_included_term,
            data: s.data,
        }
    }
}

impl From<WireRaftSnapshot> for RaftSnapshot {
    fn from(w: WireRaftSnapshot) -> Self {
        Self {
            last_included_index: w.last_included_index,
            last_included_term: w.last_included_term,
            data: w.data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireRaftMessage {
    PreVoteRequest {
        next_term: u64,
        candidate_id: u64,
        last_log_index: u64,
        last_log_term: u64,
    },
    PreVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    RequestVote {
        term: u64,
        candidate_id: u64,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    AppendEntries {
        term: u64,
        leader_id: u64,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<WireLogEntry>,
        leader_commit: u64,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
        match_index: u64,
    },
    InstallSnapshot {
        term: u64,
        leader_id: u64,
        snapshot_index: u64,
        snapshot_term: u64,
        data: Vec<u8>,
    },
    InstallSnapshotResponse {
        term: u64,
        success: bool,
    },
}

impl From<RaftMessage<Vec<u8>>> for WireRaftMessage {
    fn from(msg: RaftMessage<Vec<u8>>) -> Self {
        match msg {
            RaftMessage::PreVoteRequest {
                next_term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => WireRaftMessage::PreVoteRequest {
                next_term,
                candidate_id,
                last_log_index,
                last_log_term,
            },
            RaftMessage::PreVoteResponse { term, vote_granted } => {
                WireRaftMessage::PreVoteResponse { term, vote_granted }
            }
            RaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => WireRaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            },
            RaftMessage::RequestVoteResponse { term, vote_granted } => {
                WireRaftMessage::RequestVoteResponse { term, vote_granted }
            }
            RaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => WireRaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries: entries.into_iter().map(WireLogEntry::from).collect(),
                leader_commit,
            },
            RaftMessage::AppendEntriesResponse {
                term,
                success,
                match_index,
            } => WireRaftMessage::AppendEntriesResponse {
                term,
                success,
                match_index,
            },
            RaftMessage::InstallSnapshot {
                term,
                leader_id,
                snapshot_index,
                snapshot_term,
                data,
            } => WireRaftMessage::InstallSnapshot {
                term,
                leader_id,
                snapshot_index,
                snapshot_term,
                data,
            },
            RaftMessage::InstallSnapshotResponse { term, success } => {
                WireRaftMessage::InstallSnapshotResponse { term, success }
            }
        }
    }
}

impl From<WireRaftMessage> for RaftMessage<Vec<u8>> {
    fn from(wire: WireRaftMessage) -> Self {
        match wire {
            WireRaftMessage::PreVoteRequest {
                next_term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => RaftMessage::PreVoteRequest {
                next_term,
                candidate_id,
                last_log_index,
                last_log_term,
            },
            WireRaftMessage::PreVoteResponse { term, vote_granted } => {
                RaftMessage::PreVoteResponse { term, vote_granted }
            }
            WireRaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => RaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            },
            WireRaftMessage::RequestVoteResponse { term, vote_granted } => {
                RaftMessage::RequestVoteResponse { term, vote_granted }
            }
            WireRaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => RaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries: entries.into_iter().map(LogEntry::from).collect(),
                leader_commit,
            },
            WireRaftMessage::AppendEntriesResponse {
                term,
                success,
                match_index,
            } => RaftMessage::AppendEntriesResponse {
                term,
                success,
                match_index,
            },
            WireRaftMessage::InstallSnapshot {
                term,
                leader_id,
                snapshot_index,
                snapshot_term,
                data,
            } => RaftMessage::InstallSnapshot {
                term,
                leader_id,
                snapshot_index,
                snapshot_term,
                data,
            },
            WireRaftMessage::InstallSnapshotResponse { term, success } => {
                RaftMessage::InstallSnapshotResponse { term, success }
            }
        }
    }
}

pub fn serialize(msg: &RaftMessage<Vec<u8>>) -> Result<Vec<u8>, postcard::Error> {
    let wire = WireRaftMessage::from(msg.clone());
    postcard::to_allocvec(&wire)
}

pub fn deserialize(bytes: &[u8]) -> Result<RaftMessage<Vec<u8>>, postcard::Error> {
    let wire: WireRaftMessage = postcard::from_bytes(bytes)?;
    Ok(RaftMessage::from(wire))
}
