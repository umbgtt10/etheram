// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn new_given_four_validators_returns_four_nodes() {
    // Arrange & Act
    let cluster = IbftCluster::new(validators(), vec![]);

    // Assert
    assert_eq!(cluster.node_count(), 4);
}

#[test]
fn step_node_no_events_returns_false() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);

    // Act & Assert
    assert!(!cluster.step(0));
}

#[test]
fn fire_timer_on_proposer_then_step_returns_true() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);

    // Act & Assert
    assert!(cluster.step(0));
}

#[test]
fn fire_timer_on_non_proposer_drain_does_not_increment_height() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.fire_timer(1, TimerEvent::ProposeBlock);

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}
