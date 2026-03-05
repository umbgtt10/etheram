// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message::RaftMessage;
use crate::brain::protocol::message_source::MessageSource;
use crate::brain::protocol::unified_message::Message;
use crate::collections::action_collection::ActionCollection;
use crate::context::context_dto::RaftContext;
use crate::implementations::raft::client;
use crate::implementations::raft::election;
use crate::implementations::raft::replication;
use crate::implementations::raft::snapshot;
use crate::incoming::external_interface::client_request::RaftClientRequest;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::marker::PhantomData;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::types::{ClientId, PeerId};

pub struct RaftProtocol<P> {
    pub pre_votes_received: BTreeSet<PeerId>,
    pub votes_received: BTreeSet<PeerId>,
    pub pending_client_entries: BTreeMap<u64, ClientId>,
    _phantom: PhantomData<P>,
}

impl<P> RaftProtocol<P> {
    pub fn new() -> Self {
        Self {
            pre_votes_received: BTreeSet::new(),
            votes_received: BTreeSet::new(),
            pending_client_entries: BTreeMap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<P> Default for RaftProtocol<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Clone + 'static + From<Vec<u8>> + AsRef<[u8]>> ConsensusProtocol for RaftProtocol<P> {
    type Message = Message<P>;
    type MessageSource = MessageSource;
    type Action = RaftAction<P>;
    type Context = RaftContext<P>;
    type ActionCollection = ActionCollection<RaftAction<P>>;

    fn handle_message(
        &mut self,
        source: &Self::MessageSource,
        message: &Self::Message,
        ctx: &Self::Context,
    ) -> Self::ActionCollection {
        let raw = dispatch(self, source, message, ctx);
        let mut actions = ActionCollection::new();
        for action in raw {
            actions.push(action);
        }
        actions
    }
}

fn dispatch<P: Clone + 'static + From<Vec<u8>> + AsRef<[u8]>>(
    protocol: &mut RaftProtocol<P>,
    source: &MessageSource,
    message: &Message<P>,
    ctx: &RaftContext<P>,
) -> Vec<RaftAction<P>> {
    match message {
        Message::Timer(event) => handle_timer(protocol, ctx, *event),
        Message::Client(request) => {
            let client_id = match source {
                MessageSource::Client(id) => *id,
                _ => return Vec::new(),
            };
            handle_client(protocol, ctx, client_id, request)
        }
        Message::Peer(raft_msg) => {
            let from = match source {
                MessageSource::Peer(peer_id) => *peer_id,
                _ => return Vec::new(),
            };
            handle_peer(protocol, ctx, from, raft_msg)
        }
    }
}

fn handle_timer<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    event: RaftTimerEvent,
) -> Vec<RaftAction<P>> {
    match event {
        RaftTimerEvent::ElectionTimeout | RaftTimerEvent::PreVoteTimeout => {
            election::handle_election_timeout(protocol, ctx)
        }
        RaftTimerEvent::Heartbeat => replication::handle_heartbeat(ctx),
    }
}

fn handle_client<P: Clone + From<Vec<u8>>>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    client_id: ClientId,
    request: &RaftClientRequest,
) -> Vec<RaftAction<P>> {
    match request {
        RaftClientRequest::Command(bytes) => {
            client::handle_client_command(protocol, ctx, client_id, P::from(bytes.clone()))
        }
        RaftClientRequest::Query(key) => client::handle_client_query(ctx, client_id, key),
    }
}

fn handle_peer<P: Clone + 'static + AsRef<[u8]>>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    msg: &RaftMessage<P>,
) -> Vec<RaftAction<P>> {
    match msg {
        RaftMessage::PreVoteRequest {
            next_term,
            candidate_id,
            last_log_index,
            last_log_term,
        } => election::handle_pre_vote_request(
            ctx,
            from,
            *next_term,
            *candidate_id,
            *last_log_index,
            *last_log_term,
        ),
        RaftMessage::PreVoteResponse { vote_granted, .. } => {
            election::handle_pre_vote_response(protocol, ctx, from, *vote_granted)
        }
        RaftMessage::RequestVote {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        } => election::handle_request_vote(
            ctx,
            from,
            *term,
            *candidate_id,
            *last_log_index,
            *last_log_term,
        ),
        RaftMessage::RequestVoteResponse { term, vote_granted } => {
            election::handle_request_vote_response(protocol, ctx, from, *term, *vote_granted)
        }
        RaftMessage::AppendEntries {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        } => replication::handle_append_entries(
            protocol,
            ctx,
            from,
            *term,
            *leader_id,
            *prev_log_index,
            *prev_log_term,
            entries.clone(),
            *leader_commit,
        ),
        RaftMessage::AppendEntriesResponse {
            term,
            success,
            match_index,
        } => replication::handle_append_entries_response(
            protocol,
            ctx,
            from,
            *term,
            *success,
            *match_index,
        ),
        RaftMessage::InstallSnapshot {
            term,
            leader_id,
            snapshot_index,
            snapshot_term,
            data,
        } => snapshot::handle_install_snapshot(
            protocol,
            ctx,
            from,
            *term,
            *leader_id,
            *snapshot_index,
            *snapshot_term,
            data.clone(),
        ),
        RaftMessage::InstallSnapshotResponse { term, success } => {
            snapshot::handle_install_snapshot_response(protocol, ctx, from, *term, *success)
        }
    }
}
