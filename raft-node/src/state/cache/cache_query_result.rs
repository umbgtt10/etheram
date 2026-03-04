// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::node_role::NodeRole;
use alloc::collections::BTreeMap;
use etheram_core::types::PeerId;

#[derive(Debug, Clone)]
pub enum RaftCacheQueryResult {
    CommitIndex(u64),
    LastApplied(u64),
    Role(NodeRole),
    LeaderId(Option<PeerId>),
    MatchIndex(u64),
    NextIndex(u64),
    AllMatchIndex(BTreeMap<PeerId, u64>),
    AllNextIndex(BTreeMap<PeerId, u64>),
}
