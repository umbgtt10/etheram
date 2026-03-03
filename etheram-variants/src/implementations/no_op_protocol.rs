// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_core::consensus_protocol::ConsensusProtocol;
use core::marker::PhantomData;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::collections::action_collection::ActionCollection;
use etheram::context::context_dto::Context;

pub struct NoOpProtocol<M> {
    _phantom: PhantomData<M>,
}

impl<M> NoOpProtocol<M> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<M> ConsensusProtocol for NoOpProtocol<M> {
    type Message = Message<M>;
    type MessageSource = MessageSource;
    type Action = Action<M>;
    type Context = Context;
    type ActionCollection = ActionCollection<Action<M>>;
    fn handle_message(
        &mut self,
        _source: &MessageSource,
        _message: &Self::Message,
        _ctx: &Self::Context,
    ) -> Self::ActionCollection {
        ActionCollection::new()
    }
}

impl<M> Default for NoOpProtocol<M> {
    fn default() -> Self {
        Self::new()
    }
}
