// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::super::prepared_certificate::PreparedCertificate;
use super::super::signature_scheme::SignatureBytes;
use super::IbftProtocol;
use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::block::BLOCK_GAS_LIMIT;
use crate::common_types::transaction::Transaction;
use crate::common_types::types::Address;
use crate::common_types::types::Gas;
use crate::common_types::types::Hash;
use crate::common_types::types::Height;
use crate::context::context_dto::Context;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

impl IbftProtocol {
    pub(super) fn valid_pre_prepare(
        &self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block: &Block,
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let expected_proposer = self
            .validator_set
            .get_proposer_for_round(height, self.current_round);
        height == ctx.current_height
            && round == self.current_round
            && *sender_id == expected_proposer
            && !self.is_malicious_sender_for_height(source, height)
            && block.height == height
            && self.valid_block(block, ctx)
            && !self.prepare_sent
    }

    pub(super) fn trackable_pre_prepare(
        &self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block: &Block,
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let expected_proposer = self
            .validator_set
            .get_proposer_for_round(height, self.current_round);
        height == ctx.current_height
            && round == self.current_round
            && *sender_id == expected_proposer
            && block.height == height
            && self.valid_block(block, ctx)
    }

    pub(super) fn valid_block(&self, block: &Block, ctx: &Context) -> bool {
        if block.state_root != ctx.state_root {
            return false;
        }
        if !Self::valid_block_gas(block) {
            return false;
        }
        if !Self::valid_transaction_ordering(&block.transactions) {
            return false;
        }
        if !Self::valid_transactions(&block.transactions, &ctx.accounts) {
            return false;
        }
        let (post_state_root, receipts_root) = self.execute_and_compute_commitments(block, ctx);
        block.post_state_root == post_state_root && block.receipts_root == receipts_root
    }

    fn valid_block_gas(block: &Block) -> bool {
        if block.gas_limit != BLOCK_GAS_LIMIT {
            return false;
        }
        let total_gas: Gas = block.transactions.iter().map(|t| t.gas_limit).sum();
        total_gas <= BLOCK_GAS_LIMIT
    }

    fn valid_transaction_ordering(transactions: &[Transaction]) -> bool {
        transactions.windows(2).all(|w| w[0] >= w[1])
    }

    fn valid_transactions(
        transactions: &[Transaction],
        accounts: &BTreeMap<Address, Account>,
    ) -> bool {
        let mut working_accounts = accounts.clone();
        for tx in transactions {
            let from_account = match working_accounts.get(&tx.from) {
                Some(account)
                    if account.balance >= tx.value
                        && account.nonce == tx.nonce
                        && tx.gas_limit > 0
                        && tx.gas_limit <= super::MAX_GAS_LIMIT
                        && tx.gas_price > 0 =>
                {
                    account.clone()
                }
                _ => return false,
            };
            working_accounts.insert(
                tx.from,
                Account {
                    balance: from_account.balance.saturating_sub(tx.value),
                    nonce: from_account.nonce + 1,
                },
            );
            let to_account = working_accounts
                .get(&tx.to)
                .cloned()
                .unwrap_or(Account::empty());
            working_accounts.insert(
                tx.to,
                Account {
                    balance: to_account.balance.saturating_add(tx.value),
                    nonce: to_account.nonce,
                },
            );
        }
        true
    }

    pub(super) fn valid_prepare(
        &self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block_hash: Hash,
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        height == ctx.current_height
            && round == self.current_round
            && self.validator_set.contains(*sender_id)
            && !self.is_malicious_sender_for_height(source, height)
            && !self.is_rejected_block_hash(height, round, block_hash)
            && self.pending_block_hash() == Some(block_hash)
    }

    pub(super) fn valid_commit(
        &self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block_hash: Hash,
        sender_signature: &SignatureBytes,
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let commit_payload = Self::commit_commitment_payload(height, round, &block_hash);
        height == ctx.current_height
            && round == self.current_round
            && self.validator_set.contains(*sender_id)
            && !self.is_malicious_sender_for_height(source, height)
            && !self.is_rejected_block_hash(height, round, block_hash)
            && self.pending_block_hash() == Some(block_hash)
            && self
                .signature_scheme
                .verify_for_peer(&commit_payload, sender_signature, *sender_id)
    }

    pub(super) fn valid_view_change(
        &self,
        source: &MessageSource,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let prepared_certificate_valid =
            prepared_certificate
                .as_ref()
                .is_none_or(|prepared_certificate| {
                    self.valid_prepared_certificate(prepared_certificate, height, round)
                });
        self.validator_set.contains(*sender_id)
            && height == ctx.current_height
            && round >= self.current_round
            && !self.is_malicious_sender_for_height(source, height)
            && prepared_certificate_valid
    }

    pub(super) fn valid_prepared_certificate(
        &self,
        prepared_certificate: &PreparedCertificate,
        height: Height,
        round: u64,
    ) -> bool {
        let unique_signers: BTreeSet<PeerId> = prepared_certificate
            .signed_prepares
            .iter()
            .map(|(p, _)| *p)
            .collect();
        let no_duplicate_signers =
            unique_signers.len() == prepared_certificate.signed_prepares.len();
        let signers_are_validators = unique_signers
            .iter()
            .all(|signer| self.validator_set.contains(*signer));
        let payload: Vec<u8> = Self::prepare_commitment_payload(
            prepared_certificate.height,
            prepared_certificate.round,
            &prepared_certificate.block_hash,
        );
        let valid_sig_count = prepared_certificate
            .signed_prepares
            .iter()
            .filter(|(peer_id, sig)| {
                self.signature_scheme
                    .verify_for_peer(&payload, sig, *peer_id)
            })
            .count();
        prepared_certificate.height == height
            && prepared_certificate.round <= round
            && no_duplicate_signers
            && unique_signers.len() >= self.validator_set.quorum_size()
            && signers_are_validators
            && valid_sig_count >= self.validator_set.quorum_size()
    }

    pub(super) fn valid_new_view(
        &self,
        source: &MessageSource,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        view_change_senders: &[u64],
        ctx: &Context,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let expected = self.validator_set.get_proposer_for_round(height, round);
        let unique_view_change_senders: BTreeSet<PeerId> =
            view_change_senders.iter().copied().collect();
        let no_duplicate_view_change_senders =
            unique_view_change_senders.len() == view_change_senders.len();
        let all_senders_are_validators = unique_view_change_senders
            .iter()
            .all(|sender| self.validator_set.contains(*sender));
        let prepared_certificate_valid =
            prepared_certificate
                .as_ref()
                .is_none_or(|prepared_certificate| {
                    self.valid_prepared_certificate(prepared_certificate, height, round)
                });
        *sender_id == expected
            && height == ctx.current_height
            && round >= self.current_round
            && !self.is_malicious_sender_for_height(source, height)
            && unique_view_change_senders.len() >= self.validator_set.quorum_size()
            && no_duplicate_view_change_senders
            && all_senders_are_validators
            && prepared_certificate_valid
    }
}
