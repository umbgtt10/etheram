// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;

pub trait SyncStorage {
    fn apply_synced_blocks(&self, blocks: &[Block]);
}
