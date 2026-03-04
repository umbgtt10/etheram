// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::node_role::NodeRole;
use etheram_core::types::{ClientId, PeerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RaftEventLevel {
    None,
    Essential,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftActionKind {
    SetTerm,
    SetVotedFor,
    AppendEntries,
    TruncateLogFrom,
    SaveSnapshot,
    AdvanceCommitIndex,
    TransitionRole,
    SetLeaderId,
    UpdateMatchIndex { peer_id: PeerId },
    UpdateNextIndex { peer_id: PeerId },
    SendMessage { to: PeerId },
    BroadcastMessage,
    ScheduleTimeout,
    ApplyToStateMachine,
    QueryStateMachine,
    RestoreFromSnapshot,
    SendClientResponse { client_id: ClientId },
    Log,
}

pub fn action_kind<P>(action: &RaftAction<P>) -> RaftActionKind {
    match action {
        RaftAction::SetTerm(_) => RaftActionKind::SetTerm,
        RaftAction::SetVotedFor(_) => RaftActionKind::SetVotedFor,
        RaftAction::AppendEntries(_) => RaftActionKind::AppendEntries,
        RaftAction::TruncateLogFrom(_) => RaftActionKind::TruncateLogFrom,
        RaftAction::SaveSnapshot(_) => RaftActionKind::SaveSnapshot,
        RaftAction::AdvanceCommitIndex(_) => RaftActionKind::AdvanceCommitIndex,
        RaftAction::TransitionRole(_) => RaftActionKind::TransitionRole,
        RaftAction::SetLeaderId(_) => RaftActionKind::SetLeaderId,
        RaftAction::UpdateMatchIndex { peer_id, .. } => {
            RaftActionKind::UpdateMatchIndex { peer_id: *peer_id }
        }
        RaftAction::UpdateNextIndex { peer_id, .. } => {
            RaftActionKind::UpdateNextIndex { peer_id: *peer_id }
        }
        RaftAction::SendMessage { to, .. } => RaftActionKind::SendMessage { to: *to },
        RaftAction::BroadcastMessage { .. } => RaftActionKind::BroadcastMessage,
        RaftAction::ScheduleTimeout { .. } => RaftActionKind::ScheduleTimeout,
        RaftAction::ApplyToStateMachine { .. } => RaftActionKind::ApplyToStateMachine,
        RaftAction::QueryStateMachine { .. } => RaftActionKind::QueryStateMachine,
        RaftAction::RestoreFromSnapshot(_) => RaftActionKind::RestoreFromSnapshot,
        RaftAction::SendClientResponse { client_id, .. } => RaftActionKind::SendClientResponse {
            client_id: *client_id,
        },
        RaftAction::Log(_) => RaftActionKind::Log,
    }
}

pub trait RaftObserver {
    fn min_level(&self) -> RaftEventLevel;

    fn node_started(&mut self, peer_id: PeerId);

    fn message_received(&mut self, peer_id: PeerId, source: &MessageSource);

    fn context_built(&mut self, peer_id: PeerId, term: u64, role: NodeRole, log_length: usize);

    fn action_emitted(&mut self, peer_id: PeerId, kind: &RaftActionKind);

    fn mutation_applied(&mut self, peer_id: PeerId, kind: &RaftActionKind);

    fn output_executed(&mut self, peer_id: PeerId, kind: &RaftActionKind);

    fn step_completed(&mut self, peer_id: PeerId, processed: bool);
}
