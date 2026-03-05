// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy_shared_state::EmbassySharedState;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::wal_writer::WalWriter;
use etheram_node::implementations::shared_state::SharedState;

pub struct CapturedWalWriter {
    state: EmbassySharedState<Option<ConsensusWal>>,
    last_cert: EmbassySharedState<Option<PreparedCertificate>>,
}

impl CapturedWalWriter {
    pub fn new(
        state: EmbassySharedState<Option<ConsensusWal>>,
        last_cert: EmbassySharedState<Option<PreparedCertificate>>,
    ) -> Self {
        Self { state, last_cert }
    }
}

impl WalWriter for CapturedWalWriter {
    fn write(&mut self, wal: &ConsensusWal) {
        if let Some(cert) = &wal.prepared_certificate {
            self.last_cert.with_mut(|c| *c = Some(cert.clone()));
        }
        self.state.with_mut(|s| *s = Some(wal.clone()));
    }
}
