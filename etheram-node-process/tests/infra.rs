// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub mod transport {
    pub mod transport_backend {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/transport/transport_backend.rs"
        ));
    }

    pub mod grpc_transport {
        pub mod grpc_transport_proto {
            pub mod wire {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/infra/transport/grpc_transport/generated/etheram.transport.rs"
                ));
            }
        }

        pub mod wire_ibft_message {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/wire_ibft_message.rs"
            ));
        }

        pub mod wire_node_message {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/wire_node_message.rs"
            ));
        }

        pub mod grpc_transport_bus {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/grpc_transport_bus.rs"
            ));
        }

        pub mod sync_bus {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/sync_bus.rs"
            ));
        }

        pub mod grpc_transport_incoming {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/grpc_transport_incoming.rs"
            ));
        }

        pub mod grpc_transport_outgoing {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/grpc_transport/grpc_transport_outgoing.rs"
            ));
        }
    }

    pub mod partitionable_transport {
        pub mod partition_table {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/infra/transport/partitionable_transport/partition_table.rs"
            ));
        }
    }
}

pub mod sync {
    pub mod sync_import {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/sync/sync_import.rs"
        ));
    }

    pub mod sync_message {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/sync/sync_message.rs"
        ));
    }

    pub mod sync_sender {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/sync/sync_sender.rs"
        ));
    }

    pub mod sync_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/sync/sync_state.rs"
        ));
    }
}

pub mod storage {
    pub mod injected_storage {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/infra/storage/injected_storage.rs"
        ));
    }
}
