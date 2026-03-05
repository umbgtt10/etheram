// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_export]
macro_rules! define_client_facade {
    (
        $max_nodes:path,
        $request_ty:ty,
        $response_ty:ty,
        $channel_hub:path,
        $udp_send_fn:path,
        $udp_receive_fn:path
    ) => {
        pub fn submit_ei_request(
            node_index: usize,
            client_id: etheram_core::types::ClientId,
            request: $request_ty,
        ) {
            submit_impl(node_index, client_id, request);
        }

        pub fn submit_ei_to_all_nodes(
            client_id: etheram_core::types::ClientId,
            request: $request_ty,
        ) {
            for node_index in 0..$max_nodes {
                submit_impl(node_index, client_id, request.clone());
            }
        }

        pub async fn await_ei_response(
            node_index: usize,
        ) -> (etheram_core::types::ClientId, $response_ty) {
            await_impl(node_index).await
        }

        #[cfg(feature = "channel-external-interface")]
        fn submit_impl(
            node_index: usize,
            client_id: etheram_core::types::ClientId,
            request: $request_ty,
        ) {
            $channel_hub.send_request(node_index, client_id, request);
        }

        #[cfg(feature = "udp-external-interface")]
        fn submit_impl(
            node_index: usize,
            client_id: etheram_core::types::ClientId,
            request: $request_ty,
        ) {
            $udp_send_fn(node_index, client_id, request);
        }

        #[cfg(feature = "channel-external-interface")]
        async fn await_impl(node_index: usize) -> (etheram_core::types::ClientId, $response_ty) {
            $channel_hub.receive_response(node_index).await
        }

        #[cfg(feature = "udp-external-interface")]
        async fn await_impl(node_index: usize) -> (etheram_core::types::ClientId, $response_ty) {
            $udp_receive_fn(node_index).await
        }
    };
}
