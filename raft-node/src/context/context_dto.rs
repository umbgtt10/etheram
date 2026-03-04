// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::log_entry::LogEntry;
use crate::common_types::node_role::NodeRole;
use crate::common_types::snapshot::RaftSnapshot;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub struct RaftContext<P> {
    pub peer_id: PeerId,
    pub current_term: u64,
    pub voted_for: Option<PeerId>,
    pub log: Vec<LogEntry<P>>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub role: NodeRole,
    pub leader_id: Option<PeerId>,
    pub peers: Vec<PeerId>,
    pub match_index: BTreeMap<PeerId, u64>,
    pub next_index: BTreeMap<PeerId, u64>,
    pub snapshot: Option<RaftSnapshot>,
}
