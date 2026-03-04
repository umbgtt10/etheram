// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::log_entry::LogEntry;
use crate::common_types::snapshot::RaftSnapshot;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

#[derive(Debug, Clone)]
pub enum RaftStorageQueryResult<P> {
    CurrentTerm(u64),
    VotedFor(Option<PeerId>),
    LogLength(u64),
    LastLogTerm(u64),
    Snapshot(Option<RaftSnapshot>),
    AllEntries(Vec<LogEntry<P>>),
    Entry(Option<LogEntry<P>>),
    Entries(Vec<LogEntry<P>>),
}
