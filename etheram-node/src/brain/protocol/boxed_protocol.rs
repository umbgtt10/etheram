// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use crate::brain::protocol::message::Message;
use crate::brain::protocol::message_source::MessageSource;
use crate::context::context_dto::Context;
use alloc::boxed::Box;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::node_common::action_collection::ActionCollection;

pub type BoxedProtocol<M> = Box<
    dyn ConsensusProtocol<
        Message = Message<M>,
        MessageSource = MessageSource,
        Action = Action<M>,
        Context = Context,
        ActionCollection = ActionCollection<Action<M>>,
    >,
>;
