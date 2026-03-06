// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::node_common::action_collection::ActionCollection;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::{Address, Balance, Gas, Nonce};
use etheram_node::context::context_dto::Context;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::state::cache::cache_update::CacheUpdate;
use proptest::prelude::*;

const MAX_GAS_LIMIT: Gas = 1_000_000;

fn arb_address() -> BoxedStrategy<Address> {
    any::<[u8; 20]>().boxed()
}

fn arb_balance() -> BoxedStrategy<Balance> {
    (0u128..=1_000_000u128).boxed()
}

fn arb_nonce() -> BoxedStrategy<Nonce> {
    (0u64..=10u64).boxed()
}

fn arb_context() -> BoxedStrategy<Context> {
    (0u64..4u64, 0u64..10u64)
        .prop_map(|(peer_id, height)| Context::new(peer_id, height, [0u8; 32]))
        .boxed()
}

fn send_client_request(
    request: ClientRequest,
    ctx: &Context,
) -> ActionCollection<Action<IbftMessage>> {
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    protocol.handle_message(&MessageSource::Client(99), &Message::Client(request), ctx)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn get_balance_returns_exactly_one_balance_response(
        ctx in arb_context(),
        addr in arb_address(),
    ) {
        // Arrange & Act
        let actions = send_client_request(ClientRequest::GetBalance(addr), &ctx);

        // Assert
        prop_assert_eq!(actions.len(), 1);
        let is_balance_response = matches!(
            actions.get(0),
            Some(Action::SendClientResponse {
                response: ClientResponse::Balance { .. },
                ..
            })
        );
        prop_assert!(is_balance_response);
    }

    #[test]
    fn get_balance_reports_exact_account_balance(
        mut ctx in arb_context(),
        addr in arb_address(),
        balance in arb_balance(),
    ) {
        // Arrange
        ctx.accounts.insert(addr, Account::new(balance));

        // Act
        let actions = send_client_request(ClientRequest::GetBalance(addr), &ctx);

        // Assert
        let reports_correct_balance = matches!(
            actions.get(0),
            Some(Action::SendClientResponse {
                response: ClientResponse::Balance { balance: b, .. },
                ..
            }) if *b == balance
        );
        prop_assert!(reports_correct_balance);
    }

    #[test]
    fn get_height_always_returns_value_matching_context(
        ctx in arb_context(),
    ) {
        // Arrange & Act
        let actions = send_client_request(ClientRequest::GetHeight, &ctx);

        // Assert
        prop_assert_eq!(actions.len(), 1);
        let height_matches_context = matches!(
            actions.get(0),
            Some(Action::SendClientResponse {
                response: ClientResponse::Height(h),
                ..
            }) if *h == ctx.current_height
        );
        prop_assert!(height_matches_context);
    }

    #[test]
    fn submit_tx_over_gas_limit_always_returns_gas_limit_exceeded(
        ctx in arb_context(),
        from in arb_address(),
        to in arb_address(),
        value in arb_balance(),
        nonce in arb_nonce(),
    ) {
        // Arrange
        let tx = Transaction::transfer(from, to, value, MAX_GAS_LIMIT + 1, 1, nonce);

        // Act
        let actions = send_client_request(ClientRequest::SubmitTransaction(tx), &ctx);

        // Assert
        let is_gas_limit_exceeded = matches!(
            actions.get(0),
            Some(Action::SendClientResponse {
                response: ClientResponse::TransactionRejected {
                    reason: TransactionRejectionReason::GasLimitExceeded,
                },
                ..
            })
        );
        prop_assert!(is_gas_limit_exceeded);
    }

    #[test]
    fn rejected_tx_never_produces_update_cache_action(
        ctx in arb_context(),
        from in arb_address(),
        to in arb_address(),
        value in arb_balance(),
        nonce in arb_nonce(),
    ) {
        // Arrange
        let tx = Transaction::transfer(from, to, value, MAX_GAS_LIMIT + 1, 1, nonce);

        // Act
        let actions = send_client_request(ClientRequest::SubmitTransaction(tx), &ctx);

        // Assert
        let cache_updates = actions
            .iter()
            .filter(|a| matches!(a, Action::UpdateCache { .. }))
            .count();
        prop_assert_eq!(cache_updates, 0);
    }

    #[test]
    fn accepted_tx_always_produces_exactly_one_add_pending(
        mut ctx in arb_context(),
        from in arb_address(),
        to in arb_address(),
        nonce in arb_nonce(),
    ) {
        // Arrange
        let tx = Transaction::transfer(from, to, 0u128, MAX_GAS_LIMIT, 1, nonce);
        ctx.accounts.insert(from, Account { balance: 0, nonce });

        // Act
        let actions = send_client_request(ClientRequest::SubmitTransaction(tx), &ctx);

        // Assert
        let add_pending_count = actions
            .iter()
            .filter(|a| matches!(a, Action::UpdateCache { update: CacheUpdate::AddPending(_) }))
            .count();
        prop_assert_eq!(add_pending_count, 1);
    }

    #[test]
    fn get_balance_unknown_address_always_returns_zero(
        ctx in arb_context(),
        addr in arb_address(),
    ) {
        // Arrange — context has no entry for addr (accounts BTreeMap starts empty)

        // Act
        let actions = send_client_request(ClientRequest::GetBalance(addr), &ctx);

        // Assert
        let returns_zero = matches!(
            actions.get(0),
            Some(Action::SendClientResponse {
                response: ClientResponse::Balance { balance: 0, .. },
                ..
            })
        );
        prop_assert!(returns_zero);
    }
}
