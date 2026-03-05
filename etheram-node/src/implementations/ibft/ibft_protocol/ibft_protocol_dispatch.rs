// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::super::ibft_message::IbftMessage;
use super::super::prepared_certificate::PreparedCertificate;
use super::super::signature_scheme::SignatureBytes;
use super::IbftProtocol;
use crate::brain::protocol::action::Action;
use crate::brain::protocol::message::Message;
use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::types::{Hash, Height};
use crate::context::context_dto::Context;
use crate::executor::outgoing::external_interface::client_response::ClientResponse;
use crate::executor::outgoing::external_interface::client_response::TransactionRejectionReason;
use crate::incoming::external_interface::client_request::ClientRequest;
use crate::incoming::timer::timer_event::TimerEvent;
use crate::state::cache::cache_update::CacheUpdate;
use etheram_core::collection::Collection;
use etheram_core::node_common::action_collection::ActionCollection;

impl IbftProtocol {
    fn empty_actions() -> ActionCollection<Action<IbftMessage>> {
        ActionCollection::new()
    }

    pub(super) fn handle_client_message(
        source: &MessageSource,
        message: &Message<IbftMessage>,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let MessageSource::Client(client_id) = source else {
            return Self::empty_actions();
        };
        let Message::Client(request) = message else {
            return Self::empty_actions();
        };
        let mut actions = ActionCollection::new();
        match request {
            ClientRequest::GetHeight => {
                actions.push(Action::SendClientResponse {
                    client_id: *client_id,
                    response: ClientResponse::Height(ctx.current_height),
                });
            }
            ClientRequest::GetBalance(address) => {
                let balance = ctx.accounts.get(address).map(|a| a.balance).unwrap_or(0);
                actions.push(Action::SendClientResponse {
                    client_id: *client_id,
                    response: ClientResponse::Balance {
                        balance,
                        height: ctx.current_height,
                    },
                });
            }
            ClientRequest::SubmitTransaction(tx) => {
                let empty = Account::empty();
                let from = ctx.accounts.get(&tx.from).unwrap_or(&empty);
                let rejection = if tx.gas_limit > super::MAX_GAS_LIMIT {
                    Some(TransactionRejectionReason::GasLimitExceeded)
                } else if from.nonce != tx.nonce {
                    Some(TransactionRejectionReason::InvalidNonce)
                } else if from.balance < tx.value {
                    Some(TransactionRejectionReason::InsufficientBalance)
                } else {
                    None
                };
                match rejection {
                    Some(reason) => {
                        actions.push(Action::SendClientResponse {
                            client_id: *client_id,
                            response: ClientResponse::TransactionRejected { reason },
                        });
                    }
                    None => {
                        actions.push(Action::UpdateCache {
                            update: CacheUpdate::AddPending(tx.clone()),
                        });
                        actions.push(Action::SendClientResponse {
                            client_id: *client_id,
                            response: ClientResponse::TransactionAccepted,
                        });
                    }
                }
            }
        }
        actions
    }

    pub(super) fn handle_timer_message(
        &mut self,
        message: &Message<IbftMessage>,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        match message {
            Message::Timer(TimerEvent::ProposeBlock) => self.handle_timer_propose_block(ctx),
            Message::Timer(TimerEvent::TimeoutRound) => self.handle_timer_timeout_round(ctx),
            _ => Self::empty_actions(),
        }
    }

    pub(super) fn handle_peer_message(
        &mut self,
        source: &MessageSource,
        message: &Message<IbftMessage>,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        match message {
            Message::Peer(IbftMessage::PrePrepare {
                sequence,
                height,
                round,
                block,
            }) => self.handle_pre_prepare_message(source, *sequence, *height, *round, block, ctx),
            Message::Peer(IbftMessage::Prepare {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            }) => self.handle_prepare_message(
                source,
                *sequence,
                *height,
                *round,
                *block_hash,
                *sender_signature,
                ctx,
            ),
            Message::Peer(IbftMessage::Commit {
                sequence,
                height,
                round,
                block_hash,
                sender_signature,
            }) => self.handle_commit_message(
                source,
                *sequence,
                *height,
                *round,
                *block_hash,
                *sender_signature,
                ctx,
            ),
            Message::Peer(IbftMessage::ViewChange {
                sequence,
                height,
                round,
                prepared_certificate,
            }) => self.handle_view_change_message(
                source,
                *sequence,
                *height,
                *round,
                prepared_certificate,
                ctx,
            ),
            Message::Peer(IbftMessage::NewView {
                sequence,
                height,
                round,
                prepared_certificate,
                view_change_senders,
            }) => {
                self.handle_new_view_message(
                    source,
                    *sequence,
                    *height,
                    *round,
                    prepared_certificate,
                    view_change_senders,
                    ctx,
                );
                Self::empty_actions()
            }
            _ => Self::empty_actions(),
        }
    }

    fn accept_peer_message(
        &mut self,
        source: &MessageSource,
        height: Height,
        message_kind: u8,
        sequence: u64,
    ) -> bool {
        if self.is_duplicate_message(source, height, message_kind, sequence) {
            return false;
        }
        if !self.valid_peer_sequence(source, sequence, message_kind) {
            return false;
        }
        self.record_peer_sequence(source, sequence, message_kind);
        self.record_seen_message(source, height, message_kind, sequence);
        true
    }

    fn handle_pre_prepare_message(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        height: Height,
        round: u64,
        block: &Block,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let msg = IbftMessage::PrePrepare {
            sequence,
            height,
            round,
            block: block.clone(),
        };
        if self.try_buffer_future_message(source, round, &msg) {
            return Self::empty_actions();
        }
        if !self.accept_peer_message(source, height, 0, sequence) {
            return Self::empty_actions();
        }
        self.handle_pre_prepare(source, height, round, block, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_prepare_message(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let msg = IbftMessage::Prepare {
            sequence,
            height,
            round,
            block_hash,
            sender_signature,
        };
        if self.try_buffer_future_message(source, round, &msg) {
            return Self::empty_actions();
        }
        if !self.accept_peer_message(source, height, 1, sequence) {
            return Self::empty_actions();
        }
        self.handle_prepare(source, height, round, block_hash, sender_signature, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_commit_message(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let msg = IbftMessage::Commit {
            sequence,
            height,
            round,
            block_hash,
            sender_signature,
        };
        if self.try_buffer_future_message(source, round, &msg) {
            return Self::empty_actions();
        }
        if !self.accept_peer_message(source, height, 2, sequence) {
            return Self::empty_actions();
        }
        self.handle_commit(source, height, round, block_hash, sender_signature, ctx)
    }

    fn handle_view_change_message(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        if !self.accept_peer_message(source, height, 3, sequence) {
            return Self::empty_actions();
        }
        self.handle_view_change(source, height, round, prepared_certificate, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_new_view_message(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        view_change_senders: &[u64],
        ctx: &Context,
    ) {
        if !self.accept_peer_message(source, height, 4, sequence) {
            return;
        }
        self.handle_new_view(
            source,
            height,
            round,
            prepared_certificate,
            view_change_senders,
            ctx,
        );
    }
}
