// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message_source::MessageSource;
use crate::brain::protocol::unified_message::Message;
use crate::context::context_dto::RaftContext;
use crate::state::raft_state::RaftState;
use etheram_core::types::PeerId;

pub trait RaftContextBuilder<P> {
    fn build(
        &self,
        state: &RaftState<P>,
        peer_id: PeerId,
        source: &MessageSource,
        message: &Message<P>,
    ) -> RaftContext<P>;
}
