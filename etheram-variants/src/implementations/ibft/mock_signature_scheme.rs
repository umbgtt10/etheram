// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::signature_scheme::SignatureBytes;
use crate::implementations::ibft::signature_scheme::SignatureScheme;
use etheram_core::types::PeerId;

pub struct MockSignatureScheme {
    _peer_id: PeerId,
}

impl MockSignatureScheme {
    pub fn new(peer_id: PeerId) -> Self {
        Self { _peer_id: peer_id }
    }
}

impl SignatureScheme for MockSignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        SignatureBytes::zeroed()
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        true
    }
}
