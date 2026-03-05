// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::Message;
use crate::brain::protocol::message_source::MessageSource;
use crate::context::context_dto::Context;
use crate::state::etheram_state::EtheramState;
use etheram_core::types::PeerId;

pub trait ContextBuilder<M> {
    fn build(
        &self,
        state: &EtheramState,
        peer_id: PeerId,
        source: &MessageSource,
        message: &Message<M>,
    ) -> Context;
}
