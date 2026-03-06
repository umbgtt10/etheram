// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_client_message::wire_client_response::WireClientResponse;
use etheram_core::types::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WireEiResponse {
    pub client_id: ClientId,
    pub response: WireClientResponse,
}
