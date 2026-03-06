// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::block::Block;
use crate::common_types::transaction::Transaction;
use crate::common_types::types::Address;
use crate::common_types::types::Balance;
use crate::common_types::types::Gas;
use crate::common_types::types::GasPrice;
use crate::common_types::types::Hash;
use crate::common_types::types::Height;
use crate::common_types::types::Nonce;
use crate::implementations::ibft::prepared_certificate::PreparedCertificate;
use crate::implementations::ibft::signature_scheme::SignatureBytes;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

#[derive(Clone)]
pub struct ConsensusWal {
    pub height: Height,
    pub round: u64,
    pub active_validators: Vec<PeerId>,
    pub scheduled_validator_updates: BTreeMap<Height, Vec<PeerId>>,
    pub pending_block: Option<Block>,
    pub observed_pre_prepares: BTreeMap<(Height, u64, PeerId), Hash>,
    pub prepared_certificate: Option<PreparedCertificate>,
    pub prepare_votes: BTreeMap<(Height, u64, Hash), Vec<PeerId>>,
    pub commit_votes: BTreeMap<(Height, u64, Hash), Vec<PeerId>>,
    pub rejected_block_hashes: Vec<(Height, u64, Hash)>,
    pub malicious_senders: Vec<(Height, PeerId)>,
    pub view_change_votes: BTreeMap<(Height, u64), Vec<PeerId>>,
    pub seen_messages: Vec<(Height, PeerId, u8, u64)>,
    pub highest_seen_sequence: BTreeMap<(PeerId, u8), u64>,
    pub prepare_sent: bool,
    pub commit_sent: bool,
    pub new_view_sent_round: Option<u64>,
    pub next_outgoing_sequence: u64,
    pub prepare_signatures: Vec<(Height, u64, PeerId, SignatureBytes)>,
}

impl ConsensusWal {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        enc_u64(&mut buf, self.height);
        enc_u64(&mut buf, self.round);
        enc_peer_vec(&mut buf, &self.active_validators);
        enc_u64(&mut buf, self.scheduled_validator_updates.len() as u64);
        for (k, v) in &self.scheduled_validator_updates {
            enc_u64(&mut buf, *k);
            enc_peer_vec(&mut buf, v);
        }
        enc_opt_block(&mut buf, self.pending_block.as_ref());
        enc_u64(&mut buf, self.observed_pre_prepares.len() as u64);
        for ((h, r, p), hash) in &self.observed_pre_prepares {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *r);
            enc_u64(&mut buf, *p);
            enc_hash(&mut buf, hash);
        }
        enc_opt_cert(&mut buf, self.prepared_certificate.as_ref());
        enc_vote_map(&mut buf, &self.prepare_votes);
        enc_vote_map(&mut buf, &self.commit_votes);
        enc_u64(&mut buf, self.rejected_block_hashes.len() as u64);
        for (h, r, hash) in &self.rejected_block_hashes {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *r);
            enc_hash(&mut buf, hash);
        }
        enc_u64(&mut buf, self.malicious_senders.len() as u64);
        for (h, p) in &self.malicious_senders {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *p);
        }
        enc_u64(&mut buf, self.view_change_votes.len() as u64);
        for ((h, r), voters) in &self.view_change_votes {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *r);
            enc_peer_vec(&mut buf, voters);
        }
        enc_u64(&mut buf, self.seen_messages.len() as u64);
        for (h, p, kind, seq) in &self.seen_messages {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *p);
            buf.push(*kind);
            enc_u64(&mut buf, *seq);
        }
        enc_u64(&mut buf, self.highest_seen_sequence.len() as u64);
        for ((p, kind), seq) in &self.highest_seen_sequence {
            enc_u64(&mut buf, *p);
            buf.push(*kind);
            enc_u64(&mut buf, *seq);
        }
        buf.push(self.prepare_sent as u8);
        buf.push(self.commit_sent as u8);
        match self.new_view_sent_round {
            None => buf.push(0),
            Some(r) => {
                buf.push(1);
                enc_u64(&mut buf, r);
            }
        }
        enc_u64(&mut buf, self.next_outgoing_sequence);
        enc_u64(&mut buf, self.prepare_signatures.len() as u64);
        for (h, r, p, sig) in &self.prepare_signatures {
            enc_u64(&mut buf, *h);
            enc_u64(&mut buf, *r);
            enc_u64(&mut buf, *p);
            buf.extend_from_slice(sig.as_bytes());
        }
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let mut c = Cursor::new(data);
        let height = c.u64()?;
        let round = c.u64()?;
        let active_validators = c.peer_vec()?;
        let svu_count = c.u64()? as usize;
        let mut scheduled_validator_updates = BTreeMap::new();
        for _ in 0..svu_count {
            let k = c.u64()?;
            let v = c.peer_vec()?;
            scheduled_validator_updates.insert(k, v);
        }
        let pending_block = c.opt_block()?;
        let opp_count = c.u64()? as usize;
        let mut observed_pre_prepares = BTreeMap::new();
        for _ in 0..opp_count {
            let h = c.u64()?;
            let r = c.u64()?;
            let p = c.u64()?;
            let hash = c.hash()?;
            observed_pre_prepares.insert((h, r, p), hash);
        }
        let prepared_certificate = c.opt_cert()?;
        let prepare_votes = c.vote_map()?;
        let commit_votes = c.vote_map()?;
        let rbh_count = c.u64()? as usize;
        let mut rejected_block_hashes = Vec::new();
        for _ in 0..rbh_count {
            let h = c.u64()?;
            let r = c.u64()?;
            let hash = c.hash()?;
            rejected_block_hashes.push((h, r, hash));
        }
        let ms_count = c.u64()? as usize;
        let mut malicious_senders = Vec::new();
        for _ in 0..ms_count {
            malicious_senders.push((c.u64()?, c.u64()?));
        }
        let vcv_count = c.u64()? as usize;
        let mut view_change_votes = BTreeMap::new();
        for _ in 0..vcv_count {
            let h = c.u64()?;
            let r = c.u64()?;
            let voters = c.peer_vec()?;
            view_change_votes.insert((h, r), voters);
        }
        let sm_count = c.u64()? as usize;
        let mut seen_messages = Vec::new();
        for _ in 0..sm_count {
            let h = c.u64()?;
            let p = c.u64()?;
            let kind = c.byte()?;
            let seq = c.u64()?;
            seen_messages.push((h, p, kind, seq));
        }
        let hss_count = c.u64()? as usize;
        let mut highest_seen_sequence = BTreeMap::new();
        for _ in 0..hss_count {
            let p = c.u64()?;
            let kind = c.byte()?;
            let seq = c.u64()?;
            highest_seen_sequence.insert((p, kind), seq);
        }
        let prepare_sent = c.byte()? != 0;
        let commit_sent = c.byte()? != 0;
        let new_view_sent_round = match c.byte()? {
            0 => None,
            _ => Some(c.u64()?),
        };
        let next_outgoing_sequence = c.u64()?;
        let ps_count = c.u64()? as usize;
        let mut prepare_signatures = Vec::with_capacity(ps_count);
        for _ in 0..ps_count {
            let h = c.u64()?;
            let r = c.u64()?;
            let p = c.u64()?;
            let sig = c.sig_bytes()?;
            prepare_signatures.push((h, r, p, sig));
        }
        Some(ConsensusWal {
            height,
            round,
            active_validators,
            scheduled_validator_updates,
            pending_block,
            observed_pre_prepares,
            prepared_certificate,
            prepare_votes,
            commit_votes,
            rejected_block_hashes,
            malicious_senders,
            view_change_votes,
            seen_messages,
            highest_seen_sequence,
            prepare_sent,
            commit_sent,
            new_view_sent_round,
            next_outgoing_sequence,
            prepare_signatures,
        })
    }
}

fn enc_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn enc_hash(buf: &mut Vec<u8>, h: &Hash) {
    buf.extend_from_slice(h);
}

fn enc_address(buf: &mut Vec<u8>, a: &Address) {
    buf.extend_from_slice(a);
}

fn enc_peer_vec(buf: &mut Vec<u8>, peers: &[PeerId]) {
    enc_u64(buf, peers.len() as u64);
    for p in peers {
        enc_u64(buf, *p);
    }
}

fn enc_transaction(buf: &mut Vec<u8>, tx: &Transaction) {
    enc_address(buf, &tx.from);
    enc_address(buf, &tx.to);
    buf.extend_from_slice(&tx.value.to_le_bytes());
    enc_u64(buf, tx.gas_limit);
    enc_u64(buf, tx.gas_price);
    enc_u64(buf, tx.nonce);
}

fn enc_block(buf: &mut Vec<u8>, block: &Block) {
    enc_u64(buf, block.height);
    enc_u64(buf, block.proposer);
    enc_u64(buf, block.transactions.len() as u64);
    for tx in &block.transactions {
        enc_transaction(buf, tx);
    }
    enc_hash(buf, &block.state_root);
    enc_hash(buf, &block.post_state_root);
    enc_hash(buf, &block.receipts_root);
}

fn enc_opt_block(buf: &mut Vec<u8>, block: Option<&Block>) {
    match block {
        None => buf.push(0),
        Some(b) => {
            buf.push(1);
            enc_block(buf, b);
        }
    }
}

fn enc_cert(buf: &mut Vec<u8>, cert: &PreparedCertificate) {
    enc_u64(buf, cert.height);
    enc_u64(buf, cert.round);
    enc_hash(buf, &cert.block_hash);
    enc_u64(buf, cert.signed_prepares.len() as u64);
    for (peer_id, sig) in &cert.signed_prepares {
        enc_u64(buf, *peer_id);
        buf.extend_from_slice(sig.as_bytes());
    }
}

fn enc_opt_cert(buf: &mut Vec<u8>, cert: Option<&PreparedCertificate>) {
    match cert {
        None => buf.push(0),
        Some(c) => {
            buf.push(1);
            enc_cert(buf, c);
        }
    }
}

fn enc_vote_map(buf: &mut Vec<u8>, map: &BTreeMap<(Height, u64, Hash), Vec<PeerId>>) {
    enc_u64(buf, map.len() as u64);
    for ((h, r, hash), voters) in map {
        enc_u64(buf, *h);
        enc_u64(buf, *r);
        enc_hash(buf, hash);
        enc_peer_vec(buf, voters);
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn byte(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            return None;
        }
        let v = self.data[self.pos];
        self.pos += 1;
        Some(v)
    }

    fn u64(&mut self) -> Option<u64> {
        if self.pos + 8 > self.data.len() {
            return None;
        }
        let b = &self.data[self.pos..self.pos + 8];
        self.pos += 8;
        Some(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn u128(&mut self) -> Option<u128> {
        if self.pos + 16 > self.data.len() {
            return None;
        }
        let b = &self.data[self.pos..self.pos + 16];
        self.pos += 16;
        Some(u128::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13],
            b[14], b[15],
        ]))
    }

    fn hash(&mut self) -> Option<Hash> {
        if self.pos + 32 > self.data.len() {
            return None;
        }
        let mut h = [0u8; 32];
        h.copy_from_slice(&self.data[self.pos..self.pos + 32]);
        self.pos += 32;
        Some(h)
    }

    fn address(&mut self) -> Option<Address> {
        if self.pos + 20 > self.data.len() {
            return None;
        }
        let mut a = [0u8; 20];
        a.copy_from_slice(&self.data[self.pos..self.pos + 20]);
        self.pos += 20;
        Some(a)
    }

    fn peer_vec(&mut self) -> Option<Vec<PeerId>> {
        let count = self.u64()? as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.u64()?);
        }
        Some(v)
    }

    fn transaction(&mut self) -> Option<Transaction> {
        let from: Address = self.address()?;
        let to: Address = self.address()?;
        let value: Balance = self.u128()?;
        let gas_limit: Gas = self.u64()?;
        let gas_price: GasPrice = self.u64()?;
        let nonce: Nonce = self.u64()?;
        Some(Transaction {
            from,
            to,
            value,
            gas_limit,
            gas_price,
            nonce,
            data: Vec::new(),
        })
    }

    fn block(&mut self) -> Option<Block> {
        let height = self.u64()?;
        let proposer = self.u64()?;
        let tx_count = self.u64()? as usize;
        let mut transactions = Vec::with_capacity(tx_count);
        for _ in 0..tx_count {
            transactions.push(self.transaction()?);
        }
        let state_root = self.hash()?;
        let post_state_root = self.hash()?;
        let receipts_root = self.hash()?;
        Some(Block {
            height,
            proposer,
            transactions,
            state_root,
            post_state_root,
            receipts_root,
        })
    }

    fn opt_block(&mut self) -> Option<Option<Block>> {
        match self.byte()? {
            0 => Some(None),
            _ => Some(Some(self.block()?)),
        }
    }

    fn sig_bytes(&mut self) -> Option<SignatureBytes> {
        if self.pos + 96 > self.data.len() {
            return None;
        }
        let bytes = &self.data[self.pos..self.pos + 96];
        self.pos += 96;
        Some(SignatureBytes::from_slice(bytes))
    }

    fn cert(&mut self) -> Option<PreparedCertificate> {
        let height = self.u64()?;
        let round = self.u64()?;
        let block_hash = self.hash()?;
        let count = self.u64()? as usize;
        let mut signed_prepares = Vec::with_capacity(count);
        for _ in 0..count {
            let peer_id = self.u64()?;
            let sig = self.sig_bytes()?;
            signed_prepares.push((peer_id, sig));
        }
        Some(PreparedCertificate {
            height,
            round,
            block_hash,
            signed_prepares,
        })
    }

    fn opt_cert(&mut self) -> Option<Option<PreparedCertificate>> {
        match self.byte()? {
            0 => Some(None),
            _ => Some(Some(self.cert()?)),
        }
    }

    fn vote_map(&mut self) -> Option<BTreeMap<(Height, u64, Hash), Vec<PeerId>>> {
        let count = self.u64()? as usize;
        let mut map = BTreeMap::new();
        for _ in 0..count {
            let h = self.u64()?;
            let r = self.u64()?;
            let hash = self.hash()?;
            let voters = self.peer_vec()?;
            map.insert((h, r, hash), voters);
        }
        Some(map)
    }
}
