// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use crate::common_types::log_entry::LogEntry;
use crate::common_types::node_role::NodeRole;
use crate::common_types::snapshot::RaftSnapshot;
use crate::executor::outgoing::external_interface::client_response::RaftClientResponse;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::string::String;
use alloc::vec::Vec;
use etheram_core::types::{ClientId, PeerId};

#[derive(Debug, Clone)]
pub enum RaftAction<P> {
    SetTerm(u64),
    SetVotedFor(Option<PeerId>),
    AppendEntries(Vec<LogEntry<P>>),
    TruncateLogFrom(u64),
    SaveSnapshot(RaftSnapshot),
    AdvanceCommitIndex(u64),
    TransitionRole(NodeRole),
    SetLeaderId(Option<PeerId>),
    UpdateMatchIndex {
        peer_id: PeerId,
        index: u64,
    },
    UpdateNextIndex {
        peer_id: PeerId,
        index: u64,
    },
    SendMessage {
        to: PeerId,
        message: RaftMessage<P>,
    },
    BroadcastMessage {
        message: RaftMessage<P>,
    },
    ScheduleTimeout {
        event: RaftTimerEvent,
        delay: u64,
    },
    ApplyToStateMachine {
        client_id: Option<ClientId>,
        entry: LogEntry<P>,
    },
    QueryStateMachine {
        client_id: ClientId,
        key: String,
    },
    RestoreFromSnapshot(Vec<u8>),
    SendClientResponse {
        client_id: ClientId,
        response: RaftClientResponse,
    },
    Log(String),
}
