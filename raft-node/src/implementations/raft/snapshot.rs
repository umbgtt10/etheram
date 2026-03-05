// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message::RaftMessage;
use crate::common_types::snapshot::RaftSnapshot;
use crate::context::context_dto::RaftContext;
use crate::implementations::raft::common;
use crate::implementations::raft::common::ELECTION_TIMEOUT_MS;
use crate::implementations::raft::raft_protocol::RaftProtocol;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub struct InstallSnapshotParams {
    pub from: PeerId,
    pub term: u64,
    pub leader_id: PeerId,
    pub snapshot_index: u64,
    pub snapshot_term: u64,
    pub data: Vec<u8>,
}

pub fn handle_install_snapshot<P: Clone>(
    _protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    params: InstallSnapshotParams,
) -> Vec<RaftAction<P>> {
    let InstallSnapshotParams {
        from,
        term,
        leader_id,
        snapshot_index,
        snapshot_term,
        data,
    } = params;
    if term < ctx.current_term {
        return vec![RaftAction::SendMessage {
            to: from,
            message: RaftMessage::InstallSnapshotResponse {
                term: ctx.current_term,
                success: false,
            },
        }];
    }

    let mut actions = Vec::new();

    if term > ctx.current_term {
        actions.extend(common::step_down::<P>(term));
    }

    actions.push(RaftAction::SetLeaderId(Some(leader_id)));
    actions.push(RaftAction::ScheduleTimeout {
        event: RaftTimerEvent::ElectionTimeout,
        delay: ELECTION_TIMEOUT_MS,
    });

    if snapshot_index <= ctx.commit_index {
        actions.push(RaftAction::SendMessage {
            to: from,
            message: RaftMessage::InstallSnapshotResponse {
                term: ctx.current_term.max(term),
                success: true,
            },
        });
        return actions;
    }

    let snapshot = RaftSnapshot {
        last_included_index: snapshot_index,
        last_included_term: snapshot_term,
        data: data.clone(),
    };
    actions.push(RaftAction::SaveSnapshot(snapshot));
    actions.push(RaftAction::TruncateLogFrom(1));
    actions.push(RaftAction::AdvanceCommitIndex(snapshot_index));
    actions.push(RaftAction::RestoreFromSnapshot(data));
    actions.push(RaftAction::SendMessage {
        to: from,
        message: RaftMessage::InstallSnapshotResponse {
            term: ctx.current_term.max(term),
            success: true,
        },
    });
    actions
}

pub fn handle_install_snapshot_response<P: Clone>(
    _protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    term: u64,
    success: bool,
) -> Vec<RaftAction<P>> {
    if term > ctx.current_term {
        return common::step_down(term);
    }
    if !success {
        return Vec::new();
    }
    let snapshot_index = ctx
        .snapshot
        .as_ref()
        .map(|s| s.last_included_index)
        .unwrap_or(0);
    vec![
        RaftAction::UpdateMatchIndex {
            peer_id: from,
            index: snapshot_index,
        },
        RaftAction::UpdateNextIndex {
            peer_id: from,
            index: snapshot_index + 1,
        },
    ]
}
