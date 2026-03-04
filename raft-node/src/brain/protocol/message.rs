// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::log_entry::LogEntry;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

#[derive(Debug, Clone)]
pub enum RaftMessage<P> {
    RequestVote {
        term: u64,
        candidate_id: PeerId,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    PreVoteRequest {
        next_term: u64,
        candidate_id: PeerId,
        last_log_index: u64,
        last_log_term: u64,
    },
    PreVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    AppendEntries {
        term: u64,
        leader_id: PeerId,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry<P>>,
        leader_commit: u64,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
        match_index: u64,
    },
    InstallSnapshot {
        term: u64,
        leader_id: PeerId,
        snapshot_index: u64,
        snapshot_term: u64,
        data: Vec<u8>,
    },
    InstallSnapshotResponse {
        term: u64,
        success: bool,
    },
}
