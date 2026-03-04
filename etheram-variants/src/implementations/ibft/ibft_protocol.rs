// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::consensus_wal::ConsensusWal;
use crate::implementations::ibft::ibft_message::IbftMessage;
use crate::implementations::ibft::prepared_certificate::PreparedCertificate;
use crate::implementations::ibft::signature_scheme::BoxedSignatureScheme;
use crate::implementations::ibft::signature_scheme::SignatureBytes;
use crate::implementations::ibft::validator_set::ValidatorSet;
use crate::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use crate::implementations::ibft::vote_tracker::VoteTracker;
use crate::implementations::ibft::wal_writer::NoOpWalWriter;
use crate::implementations::ibft::wal_writer::WalWriter;
use crate::implementations::no_op_execution_engine::NoOpExecutionEngine;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::collections::action_collection::ActionCollection;
use etheram::common_types::block::Block;
use etheram::common_types::types::Gas;
use etheram::common_types::types::{Hash, Height};
use etheram::context::context_dto::Context;
use etheram::execution::block_commitments::compute_block_commitments;
use etheram::execution::execution_engine::BoxedExecutionEngine;
use etheram::state::cache::cache_update::CacheUpdate;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::types::PeerId;

pub(crate) const MAX_GAS_LIMIT: Gas = 1_000_000;
const MAX_FUTURE_BUFFER_SIZE: usize = 100;

mod ibft_protocol_dispatch;
mod ibft_protocol_message_security;
mod ibft_protocol_validation;

pub struct IbftProtocol {
    current_height: Height,
    validator_set: ValidatorSet,
    signature_scheme: BoxedSignatureScheme,
    wal_writer: Box<dyn WalWriter>,
    prepare_tracker: VoteTracker,
    commit_tracker: VoteTracker,
    observed_pre_prepares: BTreeMap<(Height, u64, PeerId), Hash>,
    rejected_block_hashes: BTreeSet<(Height, u64, Hash)>,
    malicious_senders: BTreeSet<(Height, PeerId)>,
    seen_messages: BTreeSet<(Height, PeerId, u8, u64)>,
    highest_seen_sequence: BTreeMap<(PeerId, u8), u64>,
    current_round: u64,
    next_outgoing_sequence: u64,
    prepared_certificate: Option<PreparedCertificate>,
    pending_block: Option<Block>,
    view_change_votes: BTreeMap<(Height, u64), BTreeSet<PeerId>>,
    scheduled_validator_updates: BTreeMap<Height, Vec<PeerId>>,
    prepare_sent: bool,
    commit_sent: bool,
    new_view_sent_round: Option<u64>,
    prepare_signatures: BTreeMap<(Height, u64, PeerId), SignatureBytes>,
    future_round_buffer: Vec<(MessageSource, IbftMessage)>,
    execution_engine: BoxedExecutionEngine,
}

impl IbftProtocol {
    pub fn new(validators: Vec<PeerId>, signature_scheme: BoxedSignatureScheme) -> Self {
        Self::new_with_validator_updates(validators, signature_scheme, vec![])
    }

    pub fn new_with_validator_updates(
        validators: Vec<PeerId>,
        signature_scheme: BoxedSignatureScheme,
        validator_updates: Vec<ValidatorSetUpdate>,
    ) -> Self {
        let validator_set = ValidatorSet::new(validators);
        let quorum = validator_set.quorum_size();
        Self {
            current_height: 0,
            validator_set,
            signature_scheme,
            wal_writer: Box::new(NoOpWalWriter),
            prepare_tracker: VoteTracker::new(quorum),
            commit_tracker: VoteTracker::new(quorum),
            observed_pre_prepares: BTreeMap::new(),
            rejected_block_hashes: BTreeSet::new(),
            malicious_senders: BTreeSet::new(),
            seen_messages: BTreeSet::new(),
            highest_seen_sequence: BTreeMap::new(),
            current_round: 0,
            next_outgoing_sequence: 0,
            prepared_certificate: None,
            pending_block: None,
            view_change_votes: BTreeMap::new(),
            scheduled_validator_updates: Self::build_scheduled_updates(validator_updates),
            prepare_sent: false,
            commit_sent: false,
            new_view_sent_round: None,
            prepare_signatures: BTreeMap::new(),
            future_round_buffer: Vec::new(),
            execution_engine: Box::new(NoOpExecutionEngine),
        }
    }

    pub fn with_wal_writer(mut self, wal_writer: Box<dyn WalWriter>) -> Self {
        self.wal_writer = wal_writer;
        self
    }

    pub fn with_execution_engine(mut self, engine: BoxedExecutionEngine) -> Self {
        self.execution_engine = engine;
        self
    }

    pub fn from_wal(
        validators: Vec<PeerId>,
        signature_scheme: BoxedSignatureScheme,
        wal: ConsensusWal,
    ) -> Self {
        let current_validators = if wal.active_validators.is_empty() {
            validators
        } else {
            wal.active_validators.clone()
        };
        let validator_set = ValidatorSet::new(current_validators);
        let quorum = validator_set.quorum_size();
        let view_change_votes = wal
            .view_change_votes
            .into_iter()
            .map(|(key, voters)| (key, voters.into_iter().collect()))
            .collect();
        let seen_messages = wal.seen_messages.into_iter().collect();
        Self {
            current_height: wal.height,
            validator_set,
            signature_scheme,
            wal_writer: Box::new(NoOpWalWriter),
            prepare_tracker: VoteTracker::from_snapshot(quorum, wal.prepare_votes),
            commit_tracker: VoteTracker::from_snapshot(quorum, wal.commit_votes),
            observed_pre_prepares: wal.observed_pre_prepares,
            rejected_block_hashes: wal.rejected_block_hashes.into_iter().collect(),
            malicious_senders: wal.malicious_senders.into_iter().collect(),
            seen_messages,
            highest_seen_sequence: wal.highest_seen_sequence,
            current_round: wal.round,
            next_outgoing_sequence: wal.next_outgoing_sequence,
            prepared_certificate: wal.prepared_certificate,
            pending_block: wal.pending_block,
            view_change_votes,
            scheduled_validator_updates: wal.scheduled_validator_updates,
            prepare_sent: wal.prepare_sent,
            commit_sent: wal.commit_sent,
            new_view_sent_round: wal.new_view_sent_round,
            prepare_signatures: wal
                .prepare_signatures
                .into_iter()
                .map(|(h, r, p, s)| ((h, r, p), s))
                .collect(),
            future_round_buffer: Vec::new(),
            execution_engine: Box::new(NoOpExecutionEngine),
        }
    }

    pub fn consensus_wal(&self) -> ConsensusWal {
        let view_change_votes = self
            .view_change_votes
            .iter()
            .map(|(key, voters)| (*key, voters.iter().copied().collect()))
            .collect();
        ConsensusWal {
            height: self.current_height,
            round: self.current_round,
            active_validators: self.validator_set.validators(),
            scheduled_validator_updates: self.scheduled_validator_updates.clone(),
            pending_block: self.pending_block.clone(),
            observed_pre_prepares: self.observed_pre_prepares.clone(),
            prepared_certificate: self.prepared_certificate.clone(),
            prepare_votes: self.prepare_tracker.snapshot(),
            commit_votes: self.commit_tracker.snapshot(),
            rejected_block_hashes: self.rejected_block_hashes.iter().copied().collect(),
            malicious_senders: self.malicious_senders.iter().copied().collect(),
            view_change_votes,
            seen_messages: self.seen_messages.iter().copied().collect(),
            highest_seen_sequence: self.highest_seen_sequence.clone(),
            prepare_sent: self.prepare_sent,
            commit_sent: self.commit_sent,
            new_view_sent_round: self.new_view_sent_round,
            next_outgoing_sequence: self.next_outgoing_sequence,
            prepare_signatures: self
                .prepare_signatures
                .iter()
                .map(|((h, r, p), s)| (*h, *r, *p, *s))
                .collect(),
        }
    }

    fn build_scheduled_updates(
        validator_updates: Vec<ValidatorSetUpdate>,
    ) -> BTreeMap<Height, Vec<PeerId>> {
        validator_updates
            .into_iter()
            .filter_map(|update| {
                Self::sanitize_validators(update.validators)
                    .map(|validators| (update.target_height, validators))
            })
            .collect()
    }

    fn sanitize_validators(validators: Vec<PeerId>) -> Option<Vec<PeerId>> {
        if validators.is_empty() {
            return None;
        }
        let unique: BTreeSet<PeerId> = validators.iter().copied().collect();
        if unique.len() != validators.len() {
            return None;
        }
        Some(validators)
    }

    fn apply_validator_set_update_if_due(&mut self, next_height: Height) {
        if let Some(validators) = self.scheduled_validator_updates.remove(&next_height) {
            self.validator_set = ValidatorSet::new(validators);
            let quorum = self.validator_set.quorum_size();
            self.prepare_tracker = VoteTracker::new(quorum);
            self.commit_tracker = VoteTracker::new(quorum);
            self.view_change_votes.clear();
        }
    }

    fn reset_for_new_round(&mut self) {
        if self.prepared_certificate.is_none() {
            self.pending_block = None;
        }
        self.prepare_sent = false;
        self.commit_sent = false;
        self.observed_pre_prepares.clear();
        self.rejected_block_hashes.clear();
        self.prepare_tracker.clear();
        self.commit_tracker.clear();
    }

    fn reset_after_commit(&mut self, committed_height: Height) {
        self.current_round = 0;
        self.prepared_certificate = None;
        self.prepare_signatures.clear();
        self.view_change_votes.clear();
        self.new_view_sent_round = None;
        self.malicious_senders.clear();
        self.future_round_buffer.clear();
        self.seen_messages
            .retain(|(h, _, _, _)| *h > committed_height);
        self.reset_for_new_round();
    }

    fn try_buffer_future_message(
        &mut self,
        source: &MessageSource,
        round: u64,
        msg: &IbftMessage,
    ) -> bool {
        if round <= self.current_round {
            return false;
        }
        if self.future_round_buffer.len() < MAX_FUTURE_BUFFER_SIZE {
            self.future_round_buffer.push((*source, msg.clone()));
        }
        true
    }

    fn drain_and_replay_future_messages(
        &mut self,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let buffered: Vec<(MessageSource, IbftMessage)> =
            self.future_round_buffer.drain(..).collect();
        for (source, msg) in buffered {
            let wrapped = Message::Peer(msg);
            for action in self
                .handle_peer_message(&source, &wrapped, ctx)
                .into_inner()
            {
                actions.push(action);
            }
        }
        actions
    }

    fn current_proposer(&self, height: Height) -> u64 {
        self.validator_set
            .get_proposer_for_round(height, self.current_round)
    }

    fn has_view_change_quorum(&self, height: Height, round: u64) -> bool {
        self.view_change_votes
            .get(&(height, round))
            .is_some_and(|voters| voters.len() >= self.validator_set.quorum_size())
    }

    fn record_view_change(&mut self, height: Height, round: u64, sender: u64) {
        self.view_change_votes
            .entry((height, round))
            .or_default()
            .insert(sender);
    }

    fn pending_block_hash(&self) -> Option<Hash> {
        self.pending_block.as_ref().map(Block::compute_hash)
    }

    fn is_rejected_block_hash(&self, height: Height, round: u64, block_hash: Hash) -> bool {
        self.rejected_block_hashes
            .contains(&(height, round, block_hash))
    }

    fn is_malicious_sender_for_height(&self, source: &MessageSource, height: Height) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        self.malicious_senders.contains(&(height, *sender_id))
    }

    fn record_pre_prepare_observation_and_check_conflict(
        &mut self,
        source: &MessageSource,
        height: Height,
        round: u64,
        block_hash: Hash,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        let key = (height, round, *sender_id);
        if let Some(previous_hash) = self.observed_pre_prepares.get(&key) {
            if *previous_hash != block_hash {
                self.rejected_block_hashes
                    .insert((height, round, *previous_hash));
                self.rejected_block_hashes
                    .insert((height, round, block_hash));
                self.malicious_senders.insert((height, *sender_id));
                return true;
            }
            return false;
        }
        self.observed_pre_prepares.insert(key, block_hash);
        self.is_rejected_block_hash(height, round, block_hash)
    }

    fn next_sequence(&mut self) -> u64 {
        let sequence = self.next_outgoing_sequence;
        self.next_outgoing_sequence += 1;
        sequence
    }

    fn execute_and_compute_commitments(&self, block: &Block, ctx: &Context) -> (Hash, Hash) {
        compute_block_commitments(
            block,
            &ctx.accounts,
            &ctx.contract_storage,
            self.execution_engine.as_ref(),
        )
    }

    fn handle_timer_propose_block(
        &mut self,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let is_proposer = ctx.peer_id == self.current_proposer(ctx.current_height);
        if !is_proposer || self.prepare_sent {
            return actions;
        }
        let (mut block, needs_execution) = match (&self.prepared_certificate, &self.pending_block) {
            (Some(cert), Some(b)) if b.compute_hash() == cert.block_hash => (b.clone(), false),
            _ => (
                Block::new(
                    ctx.current_height,
                    ctx.peer_id,
                    ctx.pending_txs.clone(),
                    ctx.state_root,
                ),
                true,
            ),
        };
        if needs_execution {
            let (post_state_root, receipts_root) =
                self.execute_and_compute_commitments(&block, ctx);
            block.post_state_root = post_state_root;
            block.receipts_root = receipts_root;
        }
        let block_hash = block.compute_hash();
        self.pending_block = Some(block.clone());
        self.prepare_sent = true;
        self.prepare_tracker.record(
            ctx.current_height,
            self.current_round,
            block_hash,
            ctx.peer_id,
        );
        let prepare_commitment =
            Self::prepare_commitment_payload(ctx.current_height, self.current_round, &block_hash);
        let self_sig = self.signature_scheme.sign(&prepare_commitment);
        self.prepare_signatures.insert(
            (ctx.current_height, self.current_round, ctx.peer_id),
            self_sig,
        );
        actions.push(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare {
                sequence: self.next_sequence(),
                height: ctx.current_height,
                round: self.current_round,
                block,
            },
        });
        actions.push(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                sequence: self.next_sequence(),
                height: ctx.current_height,
                round: self.current_round,
                block_hash,
                sender_signature: self_sig,
            },
        });
        actions
    }

    fn handle_timer_timeout_round(
        &mut self,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        self.current_round += 1;
        self.new_view_sent_round = None;
        self.reset_for_new_round();
        self.record_view_change(ctx.current_height, self.current_round, ctx.peer_id);
        actions.push(Action::BroadcastMessage {
            message: IbftMessage::ViewChange {
                sequence: self.next_sequence(),
                height: ctx.current_height,
                round: self.current_round,
                prepared_certificate: self.prepared_certificate.clone(),
            },
        });
        actions
    }

    fn handle_pre_prepare(
        &mut self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block: &Block,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let block_hash = block.compute_hash();
        if self.trackable_pre_prepare(source, height, round, block, ctx)
            && self.record_pre_prepare_observation_and_check_conflict(
                source, height, round, block_hash,
            )
        {
            return actions;
        }
        if !self.valid_pre_prepare(source, height, round, block, ctx) {
            return actions;
        }
        self.pending_block = Some(block.clone());
        self.prepare_sent = true;
        let prepare_commitment = Self::prepare_commitment_payload(height, round, &block_hash);
        let self_sig = self.signature_scheme.sign(&prepare_commitment);
        self.prepare_signatures
            .insert((height, round, ctx.peer_id), self_sig);
        actions.push(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                sequence: self.next_sequence(),
                height,
                round,
                block_hash,
                sender_signature: self_sig,
            },
        });
        actions
    }

    fn handle_prepare(
        &mut self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let MessageSource::Peer(sender_id) = source else {
            return actions;
        };
        if !self.valid_prepare(source, height, round, block_hash, ctx) {
            return actions;
        }
        self.prepare_signatures
            .insert((height, round, *sender_id), sender_signature);
        self.prepare_tracker
            .record(height, round, block_hash, *sender_id);
        let quorum = self.prepare_tracker.has_quorum(height, round, block_hash);
        if quorum && !self.commit_sent {
            let voters = self.prepare_tracker.voters(height, round, block_hash);
            let signed_prepares = voters
                .into_iter()
                .map(|p| {
                    let sig = self
                        .prepare_signatures
                        .get(&(height, round, p))
                        .copied()
                        .unwrap_or_else(SignatureBytes::zeroed);
                    (p, sig)
                })
                .collect();
            self.prepared_certificate = Some(PreparedCertificate {
                height,
                round,
                block_hash,
                signed_prepares,
            });
            self.commit_sent = true;
            self.commit_tracker
                .record(height, round, block_hash, ctx.peer_id);
            let commit_commitment = Self::commit_commitment_payload(height, round, &block_hash);
            let commit_sig = self.signature_scheme.sign(&commit_commitment);
            actions.push(Action::BroadcastMessage {
                message: IbftMessage::Commit {
                    sequence: self.next_sequence(),
                    height,
                    round,
                    block_hash,
                    sender_signature: commit_sig,
                },
            });
        }
        actions
    }

    fn handle_view_change(
        &mut self,
        source: &MessageSource,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let MessageSource::Peer(sender_id) = source else {
            return actions;
        };
        if !self.valid_view_change(source, height, round, prepared_certificate, ctx) {
            return actions;
        }
        if round > self.current_round {
            self.current_round = round;
            self.new_view_sent_round = None;
            self.reset_for_new_round();
        }
        self.record_view_change(height, round, *sender_id);
        if let Some(incoming) = prepared_certificate {
            match &self.prepared_certificate {
                None => self.prepared_certificate = Some(incoming.clone()),
                Some(current) if incoming.round > current.round => {
                    self.prepared_certificate = Some(incoming.clone());
                }
                _ => {}
            }
        }
        let is_round_proposer =
            ctx.peer_id == self.validator_set.get_proposer_for_round(height, round);
        let already_sent = self.new_view_sent_round == Some(round);
        if !is_round_proposer || already_sent || !self.has_view_change_quorum(height, round) {
            return actions;
        }
        self.new_view_sent_round = Some(round);
        let view_change_senders = self
            .view_change_votes
            .get(&(height, round))
            .map(|voters| voters.iter().copied().collect())
            .unwrap_or_default();
        actions.push(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                sequence: self.next_sequence(),
                height,
                round,
                prepared_certificate: self.prepared_certificate.clone(),
                view_change_senders,
            },
        });
        actions
    }

    fn handle_new_view(
        &mut self,
        source: &MessageSource,
        height: Height,
        round: u64,
        prepared_certificate: &Option<PreparedCertificate>,
        view_change_senders: &[u64],
        ctx: &Context,
    ) {
        if !self.valid_new_view(
            source,
            height,
            round,
            prepared_certificate,
            view_change_senders,
            ctx,
        ) {
            return;
        }
        self.current_round = round;
        self.prepared_certificate = prepared_certificate.clone();
        self.new_view_sent_round = None;
        self.reset_for_new_round();
    }

    fn handle_commit(
        &mut self,
        source: &MessageSource,
        height: u64,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
        ctx: &Context,
    ) -> ActionCollection<Action<IbftMessage>> {
        let mut actions = ActionCollection::new();
        let MessageSource::Peer(sender_id) = source else {
            return actions;
        };
        if !self.valid_commit(source, height, round, block_hash, &sender_signature, ctx) {
            return actions;
        }
        self.commit_tracker
            .record(height, round, block_hash, *sender_id);
        let quorum = self.commit_tracker.has_quorum(height, round, block_hash);
        if !quorum {
            return actions;
        }
        if let Some(block) = self.pending_block.take() {
            for tx in &block.transactions {
                actions.push(Action::UpdateCache {
                    update: CacheUpdate::RemovePending(tx.clone()),
                });
            }
            actions.push(Action::ExecuteBlock {
                block: block.clone(),
            });
            actions.push(Action::StoreBlock { block });
            actions.push(Action::IncrementHeight);
            self.reset_after_commit(height);
            self.apply_validator_set_update_if_due(height + 1);
        }
        actions
    }
}

impl ConsensusProtocol for IbftProtocol {
    type Message = Message<IbftMessage>;
    type MessageSource = MessageSource;
    type Action = Action<IbftMessage>;
    type Context = Context;
    type ActionCollection = ActionCollection<Action<IbftMessage>>;

    fn handle_message(
        &mut self,
        source: &MessageSource,
        message: &Self::Message,
        ctx: &Self::Context,
    ) -> Self::ActionCollection {
        self.current_height = ctx.current_height;
        let mut actions = ActionCollection::new();
        for action in Self::handle_client_message(source, message, ctx).into_inner() {
            actions.push(action);
        }
        let round_before = self.current_round;
        for action in self.handle_timer_message(message, ctx).into_inner() {
            actions.push(action);
        }
        for action in self.handle_peer_message(source, message, ctx).into_inner() {
            actions.push(action);
        }
        if self.current_round > round_before && !self.future_round_buffer.is_empty() {
            for action in self.drain_and_replay_future_messages(ctx).into_inner() {
                actions.push(action);
            }
        }
        self.wal_writer.write(&self.consensus_wal());
        actions
    }
}
