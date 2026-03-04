// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::raft_protocol::common;
use crate::implementations::raft_protocol::raft_protocol::RaftProtocol;
use crate::implementations::raft_protocol::replication;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::types::ClientId;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::node_role::NodeRole;
use raft_node::context::context_dto::RaftContext;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;

pub fn handle_client_command<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    client_id: ClientId,
    payload: P,
) -> Vec<RaftAction<P>> {
    if ctx.role != NodeRole::Leader {
        return vec![RaftAction::SendClientResponse {
            client_id,
            response: RaftClientResponse::NotLeader(ctx.leader_id),
        }];
    }
    let new_index = common::last_log_index(ctx) + 1;
    let entry = LogEntry {
        term: ctx.current_term,
        index: new_index,
        payload,
    };
    protocol.pending_client_entries.insert(new_index, client_id);
    let mut actions = vec![RaftAction::AppendEntries(vec![entry])];
    actions.extend(replication::send_all_peers(ctx));
    actions
}

pub fn handle_client_query<P: Clone>(
    ctx: &RaftContext<P>,
    client_id: ClientId,
) -> Vec<RaftAction<P>> {
    vec![RaftAction::SendClientResponse {
        client_id,
        response: RaftClientResponse::NotLeader(ctx.leader_id),
    }]
}
