// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::TIMER_COMMAND_CAPACITY;
use crate::embassy_shared_state::EmbassySharedState;
use alloc::collections::BTreeMap;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_sync::channel::Sender;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_variants::implementations::shared_state::SharedState;

pub type TimerSender = Sender<'static, CriticalSectionRawMutex, TimerEvent, TIMER_COMMAND_CAPACITY>;

pub type TimerReceiver =
    Receiver<'static, CriticalSectionRawMutex, TimerEvent, TIMER_COMMAND_CAPACITY>;

pub struct SpawnedNode {
    pub(crate) timer_sender: TimerSender,
    pub(crate) height: EmbassySharedState<Height>,
    pub(crate) contract_storage: EmbassySharedState<BTreeMap<(Address, Hash), Hash>>,
    pub(crate) wal: EmbassySharedState<Option<ConsensusWal>>,
    pub(crate) last_cert: EmbassySharedState<Option<PreparedCertificate>>,
}

impl SpawnedNode {
    pub(crate) fn read_height(&self) -> Height {
        self.height.with(|h| *h)
    }

    pub(crate) fn read_contract_storage(&self, address: Address, slot: Hash) -> Option<Hash> {
        self.contract_storage
            .with(|entries| entries.get(&(address, slot)).copied())
    }

    pub(crate) fn read_wal(&self) -> Option<ConsensusWal> {
        self.wal.with(|w| (*w).clone())
    }

    pub(crate) fn read_last_cert(&self) -> Option<PreparedCertificate> {
        self.last_cert.with(|c| (*c).clone())
    }
}
