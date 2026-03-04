// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::standard_shared_state::StdSharedState;
use etheram::brain::protocol::action::Action;
use etheram::collections::action_collection::ActionCollection;
use etheram::executor::etheram_executor::EtheramExecutor;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_core::timer_input::TimerInput;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_etheram_variants::implementations::in_memory_external_interface::InMemoryExternalInterface;
use etheram_etheram_variants::implementations::in_memory_external_interface::InMemoryExternalInterfaceState;
use etheram_etheram_variants::implementations::in_memory_timer::InMemoryTimer;
use etheram_etheram_variants::implementations::in_memory_timer::InMemoryTimerState;
use etheram_etheram_variants::implementations::in_memory_transport::InMemoryTransport;
use etheram_etheram_variants::implementations::in_memory_transport::InMemoryTransportState;
use etheram_etheram_variants::implementations::no_op_external_interface::NoOpExternalInterface;
use etheram_etheram_variants::implementations::no_op_timer::NoOpTimer;
use etheram_etheram_variants::implementations::shared_state::SharedState;

#[test]
fn execute_outputs_broadcast_message_routes_to_all_registered_peers() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
    let sender_transport = InMemoryTransport::new(0, state.clone());
    let receiver_1 = InMemoryTransport::new(1, state.clone());
    let receiver_2 = InMemoryTransport::new(2, state.clone());
    let outgoing = OutgoingSources::new(
        Box::new(NoOpTimer),
        Box::new(NoOpExternalInterface),
        Box::new(sender_transport),
    );
    let executor = EtheramExecutor::new_with_peers(outgoing, vec![1, 2]);
    let message = IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    };
    let actions =
        ActionCollection::<Action<IbftMessage>>::from_vec(vec![Action::BroadcastMessage {
            message,
        }]);

    // Act
    executor.execute_outputs(&actions);

    // Assert
    assert!(receiver_1.poll().is_some());
    assert!(receiver_2.poll().is_some());
}

#[test]
fn execute_outputs_broadcast_message_empty_peers_silently_ignored() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
    let transport = InMemoryTransport::new(0, state);
    let outgoing = OutgoingSources::new(
        Box::new(NoOpTimer),
        Box::new(NoOpExternalInterface),
        Box::new(transport),
    );
    let executor = EtheramExecutor::new(outgoing);
    let message = IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    };
    let actions =
        ActionCollection::<Action<IbftMessage>>::from_vec(vec![Action::BroadcastMessage {
            message,
        }]);

    // Act & Assert
    executor.execute_outputs(&actions);
}

#[test]
fn execute_outputs_send_message_routes_to_specific_peer() {
    // Arrange
    let state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
    let sender_transport = InMemoryTransport::new(0, state.clone());
    let receiver = InMemoryTransport::new(1, state.clone());
    let non_recipient = InMemoryTransport::new(2, state.clone());
    let outgoing = OutgoingSources::new(
        Box::new(NoOpTimer),
        Box::new(NoOpExternalInterface),
        Box::new(sender_transport),
    );
    let executor = EtheramExecutor::new_with_peers(outgoing, vec![1, 2]);
    let message = IbftMessage::ViewChange {
        sequence: 1,
        height: 0,
        round: 1,
        prepared_certificate: None,
    };
    let actions = ActionCollection::<Action<IbftMessage>>::from_vec(vec![Action::SendMessage {
        to: 1,
        message,
    }]);

    // Act
    executor.execute_outputs(&actions);

    // Assert
    assert!(receiver.poll().is_some());
    assert!(non_recipient.poll().is_none());
}

#[test]
fn execute_outputs_schedule_timeout_schedules_timer_event() {
    // Arrange
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let scheduler = InMemoryTimer::new(0, timer_state.clone());
    let poller = InMemoryTimer::new(0, timer_state);
    let transport_state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
    let transport = InMemoryTransport::new(0, transport_state);
    let outgoing = OutgoingSources::new(
        Box::new(scheduler),
        Box::new(NoOpExternalInterface),
        Box::new(transport),
    );
    let executor = EtheramExecutor::new(outgoing);
    let actions =
        ActionCollection::<Action<IbftMessage>>::from_vec(vec![Action::ScheduleTimeout {
            event: TimerEvent::ProposeBlock,
            delay: 1000,
        }]);

    // Act
    executor.execute_outputs(&actions);

    // Assert
    assert!(matches!(poller.poll(), Some(TimerEvent::ProposeBlock)));
}

#[test]
fn execute_outputs_send_client_response_routes_to_client() {
    // Arrange
    let ei_state = StdSharedState::new(InMemoryExternalInterfaceState::new());
    let ei = InMemoryExternalInterface::new(0, ei_state.clone());
    let transport_state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
    let transport = InMemoryTransport::new(0, transport_state);
    let outgoing = OutgoingSources::new(Box::new(NoOpTimer), Box::new(ei), Box::new(transport));
    let executor = EtheramExecutor::new(outgoing);
    let actions =
        ActionCollection::<Action<IbftMessage>>::from_vec(vec![Action::SendClientResponse {
            client_id: 42,
            response: ClientResponse::TransactionAccepted,
        }]);

    // Act
    executor.execute_outputs(&actions);

    // Assert
    let responses = ei_state.with_mut(|state| state.drain_responses(42));
    assert_eq!(responses.len(), 1);
    assert!(matches!(responses[0], ClientResponse::TransactionAccepted));
}
