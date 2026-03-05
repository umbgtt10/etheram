// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use etheram_core::types::PeerId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignatureBytes([u8; 96]);

impl SignatureBytes {
    pub fn zeroed() -> Self {
        Self([0u8; 96])
    }

    pub fn from_slice(src: &[u8]) -> Self {
        let mut buf = [0u8; 96];
        let len = src.len().min(96);
        buf[..len].copy_from_slice(&src[..len]);
        Self(buf)
    }

    pub fn as_bytes(&self) -> &[u8; 96] {
        &self.0
    }
}

pub trait SignatureScheme {
    type Signature;

    fn sign(&self, data: &[u8]) -> Self::Signature;
    fn verify_for_peer(&self, data: &[u8], sig: &Self::Signature, peer: PeerId) -> bool;
}

pub type BoxedSignatureScheme = Box<dyn SignatureScheme<Signature = SignatureBytes>>;
