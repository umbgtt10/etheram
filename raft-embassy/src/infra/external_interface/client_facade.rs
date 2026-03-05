// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
#[cfg(feature = "channel-external-interface")]
use crate::infra::external_interface::channel::client_request_hub::CLIENT_REQUEST_HUB;
#[cfg(feature = "udp-external-interface")]
use crate::infra::external_interface::udp::udp_raft_external_interface::receive_udp_raft_ei_response;
#[cfg(feature = "udp-external-interface")]
use crate::infra::external_interface::udp::udp_raft_external_interface::send_udp_raft_ei_request;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;

embassy_core::define_client_facade!(
    MAX_NODES,
    RaftClientRequest,
    RaftClientResponse,
    CLIENT_REQUEST_HUB,
    send_udp_raft_ei_request,
    receive_udp_raft_ei_response
);
