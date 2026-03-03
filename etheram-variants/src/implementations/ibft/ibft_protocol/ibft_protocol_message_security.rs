// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::IbftProtocol;
use alloc::vec::Vec;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::types::Hash;
use etheram::common_types::types::Height;

impl IbftProtocol {
    pub(super) fn commit_commitment_payload(
        height: Height,
        round: u64,
        block_hash: &Hash,
    ) -> Vec<u8> {
        let mut payload = Vec::with_capacity(1 + 8 + 8 + 32);
        payload.push(2);
        payload.extend_from_slice(&height.to_le_bytes());
        payload.extend_from_slice(&round.to_le_bytes());
        payload.extend_from_slice(block_hash);
        payload
    }

    pub(super) fn prepare_commitment_payload(
        height: Height,
        round: u64,
        block_hash: &Hash,
    ) -> Vec<u8> {
        let mut payload = Vec::with_capacity(1 + 8 + 8 + 32);
        payload.push(1);
        payload.extend_from_slice(&height.to_le_bytes());
        payload.extend_from_slice(&round.to_le_bytes());
        payload.extend_from_slice(block_hash);
        payload
    }

    pub(super) fn valid_peer_sequence(
        &self,
        source: &MessageSource,
        sequence: u64,
        message_kind: u8,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        self.highest_seen_sequence
            .get(&(*sender_id, message_kind))
            .is_none_or(|highest_seen| sequence > *highest_seen)
    }

    pub(super) fn record_peer_sequence(
        &mut self,
        source: &MessageSource,
        sequence: u64,
        message_kind: u8,
    ) {
        let MessageSource::Peer(sender_id) = source else {
            return;
        };
        self.highest_seen_sequence
            .insert((*sender_id, message_kind), sequence);
    }

    pub(super) fn is_duplicate_message(
        &self,
        source: &MessageSource,
        height: Height,
        message_kind: u8,
        sequence: u64,
    ) -> bool {
        let MessageSource::Peer(sender_id) = source else {
            return false;
        };
        self.seen_messages
            .contains(&(height, *sender_id, message_kind, sequence))
    }

    pub(super) fn record_seen_message(
        &mut self,
        source: &MessageSource,
        height: Height,
        message_kind: u8,
        sequence: u64,
    ) {
        let MessageSource::Peer(sender_id) = source else {
            return;
        };
        self.seen_messages
            .insert((height, *sender_id, message_kind, sequence));
    }
}
