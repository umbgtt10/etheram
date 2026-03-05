// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_export]
macro_rules! define_channel_external_interface {
    ($request_ty:ty, $response_ty:ty, $hub:path) => {
        pub struct ChannelExternalInterface {
            node_index: usize,
        }

        impl ChannelExternalInterface {
            pub fn new(node_index: usize) -> Self {
                Self { node_index }
            }
        }

        impl etheram_core::external_interface_incoming::ExternalInterfaceIncoming
            for ChannelExternalInterface
        {
            type Request = $request_ty;

            fn poll_request(&self) -> Option<(etheram_core::types::ClientId, Self::Request)> {
                $hub.try_receive_request(self.node_index)
            }
        }

        impl etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing
            for ChannelExternalInterface
        {
            type Response = $response_ty;

            fn send_response(
                &self,
                client_id: etheram_core::types::ClientId,
                response: Self::Response,
            ) {
                $hub.try_send_response(self.node_index, client_id, response);
            }
        }
    };
}
