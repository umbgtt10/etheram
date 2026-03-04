// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftCacheQuery {
    CommitIndex,
    LastApplied,
    Role,
    LeaderId,
    MatchIndex(PeerId),
    NextIndex(PeerId),
    AllMatchIndex,
    AllNextIndex,
}
