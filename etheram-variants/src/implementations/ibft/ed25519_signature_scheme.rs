// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::signature_scheme::SignatureBytes;
use crate::implementations::ibft::signature_scheme::SignatureScheme;
use barechain_core::types::PeerId;
use ed25519_dalek::Signature;
use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;

pub struct Ed25519SignatureScheme {
    signing_key: SigningKey,
}

impl Ed25519SignatureScheme {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            signing_key: Self::key_for_peer(peer_id),
        }
    }

    fn key_for_peer(peer: PeerId) -> SigningKey {
        let peer_bytes = peer.to_le_bytes();
        let mut seed = [0u8; 32];
        for i in 0..32 {
            seed[i] = peer_bytes[i % 8].wrapping_add(i as u8);
        }
        SigningKey::from_bytes(&seed)
    }
}

impl SignatureScheme for Ed25519SignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, data: &[u8]) -> SignatureBytes {
        let sig: Signature = self.signing_key.sign(data);
        SignatureBytes::from_slice(&sig.to_bytes())
    }

    fn verify_for_peer(&self, data: &[u8], sig: &SignatureBytes, peer: PeerId) -> bool {
        let signing_key = Self::key_for_peer(peer);
        let verifying_key = signing_key.verifying_key();
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&sig.as_bytes()[..64]);
        let ed_sig = Signature::from_bytes(&sig_bytes);
        verifying_key.verify_strict(data, &ed_sig).is_ok()
    }
}
