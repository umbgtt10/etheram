// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use crate::brain::protocol::message::Message;
use crate::brain::protocol::message_source::MessageSource;
use crate::collections::action_collection::ActionCollection;
use crate::context::context_dto::Context;
use alloc::boxed::Box;
use barechain_core::consensus_protocol::ConsensusProtocol;

pub type BoxedProtocol<M> = Box<
    dyn ConsensusProtocol<
        Message = Message<M>,
        MessageSource = MessageSource,
        Action = Action<M>,
        Context = Context,
        ActionCollection = ActionCollection<Action<M>>,
    >,
>;
