// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::node_role::NodeRole;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

#[derive(Debug, Clone)]
pub enum RaftCacheUpdate {
    SetCommitIndex(u64),
    SetLastApplied(u64),
    SetRole(NodeRole),
    SetLeaderId(Option<PeerId>),
    UpdateMatchIndex(PeerId, u64),
    UpdateNextIndex(PeerId, u64),
    InitPeers(Vec<PeerId>),
}
