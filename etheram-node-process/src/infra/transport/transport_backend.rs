// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum TransportBackend {
    LocalNoOp,
    Grpc,
}

impl TransportBackend {
    pub fn from_env() -> Self {
        match std::env::var("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND") {
            Ok(value) if value.eq_ignore_ascii_case("grpc") => Self::Grpc,
            _ => Self::LocalNoOp,
        }
    }
}
