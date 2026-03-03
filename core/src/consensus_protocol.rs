// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::collection::Collection;

pub trait ConsensusProtocol {
    type Message;
    type MessageSource;
    type Action;
    type Context;
    type ActionCollection: Collection<Item = Self::Action>;

    fn handle_message(
        &mut self,
        source: &Self::MessageSource,
        message: &Self::Message,
        ctx: &Self::Context,
    ) -> Self::ActionCollection;
}
