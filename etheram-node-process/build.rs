// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

fn main() {
    println!("cargo:rerun-if-changed=src/infra/transport/grpc_transport/proto/transport.proto");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/infra/transport/grpc_transport/generated")
        .compile_protos(
            &["src/infra/transport/grpc_transport/proto/transport.proto"],
            &["src/infra/transport/grpc_transport/proto"],
        )
        .expect("failed to compile transport proto");
}
