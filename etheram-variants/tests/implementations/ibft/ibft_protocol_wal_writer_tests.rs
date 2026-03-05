// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_variants::implementations::ibft::wal_writer::WalWriter;
use std::cell::RefCell;
use std::rc::Rc;

struct SpyWalWriter {
    captured: Rc<RefCell<Option<ConsensusWal>>>,
    write_count: Rc<RefCell<usize>>,
}

impl WalWriter for SpyWalWriter {
    fn write(&mut self, wal: &ConsensusWal) {
        *self.captured.borrow_mut() = Some((*wal).clone());
        *self.write_count.borrow_mut() += 1;
    }
}

#[test]
fn with_wal_writer_is_called_after_each_handle_message() {
    // Arrange
    let captured: Rc<RefCell<Option<ConsensusWal>>> = Rc::new(RefCell::new(None));
    let write_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let spy = SpyWalWriter {
        captured: captured.clone(),
        write_count: write_count.clone(),
    };
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)))
        .with_wal_writer(Box::new(spy));
    let ctx = setup_context(0, 0);

    // Act
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(*write_count.borrow(), 1);
    let wal = captured.borrow();
    assert!(wal.is_some());
    assert_eq!(wal.as_ref().unwrap().height, 0);
}

#[test]
fn with_wal_writer_write_count_matches_handle_message_calls() {
    // Arrange
    let captured: Rc<RefCell<Option<ConsensusWal>>> = Rc::new(RefCell::new(None));
    let write_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let spy = SpyWalWriter {
        captured: captured.clone(),
        write_count: write_count.clone(),
    };
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)))
        .with_wal_writer(Box::new(spy));
    let ctx = setup_context(0, 0);

    // Act
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Assert
    assert_eq!(*write_count.borrow(), 2);
    let wal = captured.borrow();
    assert!(wal.is_some());
    assert_eq!(wal.as_ref().unwrap().height, 0);
}

#[test]
fn without_wal_writer_protocol_handles_messages_normally() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = setup_context(0, 0);

    // Act & Assert
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    assert!(!actions.is_empty());
}
