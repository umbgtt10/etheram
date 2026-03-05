// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message_source::MessageSource;
use crate::brain::protocol::unified_message::Message;
use crate::context::context_dto::RaftContext;
use alloc::boxed::Box;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::node_common::action_collection::ActionCollection;

pub type BoxedRaftProtocol<P> = Box<
    dyn ConsensusProtocol<
        Message = Message<P>,
        MessageSource = MessageSource,
        Action = RaftAction<P>,
        Context = RaftContext<P>,
        ActionCollection = ActionCollection<RaftAction<P>>,
    >,
>;
