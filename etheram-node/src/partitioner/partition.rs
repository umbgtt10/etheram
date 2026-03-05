// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use etheram_core::node_common::action_collection::ActionCollection;

#[allow(clippy::type_complexity)]
pub trait Partitioner<M> {
    fn partition(
        &self,
        actions: &ActionCollection<Action<M>>,
    ) -> (
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
        ActionCollection<Action<M>>,
    );
}
