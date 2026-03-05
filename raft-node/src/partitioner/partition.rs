// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use etheram_core::node_common::action_collection::ActionCollection;

pub trait RaftPartitioner<P> {
    fn partition(
        &self,
        actions: &ActionCollection<RaftAction<P>>,
    ) -> (
        ActionCollection<RaftAction<P>>,
        ActionCollection<RaftAction<P>>,
    );
}
