// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub mod common;
pub mod consensus_wal_tests;
pub mod ed25519_signature_scheme_tests;
pub mod ibft_protocol_client_tests;
pub mod ibft_protocol_commit_tests;
pub mod ibft_protocol_dedup_tests;
pub mod ibft_protocol_future_buffer_tests;
pub mod ibft_protocol_injection_tests;
pub mod ibft_protocol_malicious_block_tests;
pub mod ibft_protocol_persistence_tests;
pub mod ibft_protocol_pre_prepare_tests;
pub mod ibft_protocol_prepare_tests;
pub mod ibft_protocol_propose_tests;
pub mod ibft_protocol_reexecution_tests;
pub mod ibft_protocol_replay_tests;
pub mod ibft_protocol_signature_tests;
pub mod ibft_protocol_validator_set_update_tests;
pub mod ibft_protocol_view_change_tests;
pub mod ibft_protocol_wal_writer_tests;
pub mod mock_signature_scheme_tests;
pub mod signature_bytes_tests;
pub mod validator_set_tests;
pub mod vote_tracker_tests;
