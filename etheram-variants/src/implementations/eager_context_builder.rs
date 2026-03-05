// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::context::context_builder::ContextBuilder;
use etheram_node::context::context_dto::Context;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::state::etheram_state::EtheramState;

pub struct EagerContextBuilder;

impl EagerContextBuilder {
    pub fn new() -> Self {
        Self
    }
    pub fn build<M>(
        &self,
        state: &EtheramState,
        peer_id: PeerId,
        _source: &MessageSource,
        message: &Message<M>,
    ) -> Context {
        let current_height = state.query_height();
        let pending_txs = state.query_pending();
        let mut addresses_to_query = Vec::new();
        for tx in &pending_txs {
            addresses_to_query.push(tx.from);
            addresses_to_query.push(tx.to);
        }
        if let Message::Client(ClientRequest::GetBalance(addr)) = message {
            addresses_to_query.push(*addr);
        }
        if let Message::Client(ClientRequest::SubmitTransaction(tx)) = message {
            addresses_to_query.push(tx.from);
        }
        addresses_to_query.sort();
        addresses_to_query.dedup();
        let mut accounts = BTreeMap::new();
        for addr in addresses_to_query {
            if let Some(account) = state.query_account(addr) {
                if account.balance > 0 || account.nonce > 0 {
                    accounts.insert(addr, account);
                }
            }
        }
        let state_root = state.query_state_root();
        let contract_storage = state.snapshot_contract_storage();
        Context {
            peer_id,
            current_height,
            state_root,
            accounts,
            contract_storage,
            pending_txs,
        }
    }
}

impl Default for EagerContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> ContextBuilder<M> for EagerContextBuilder {
    fn build(
        &self,
        state: &EtheramState,
        peer_id: PeerId,
        source: &MessageSource,
        message: &Message<M>,
    ) -> Context {
        self.build(state, peer_id, source, message)
    }
}
