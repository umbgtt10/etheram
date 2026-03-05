// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::brain::protocol::action::Action;
use etheram_node::collections::action_collection::ActionCollection;
use etheram_node::common_types::block::Block;
use etheram_node::partitioner::partition::Partitioner;
use etheram_variants::implementations::type_based_partitioner::TypeBasedPartitioner;

#[test]
fn partition_mutation_action_goes_to_mutations_only() {
    // Arrange
    let partitioner = TypeBasedPartitioner::new();
    let actions: ActionCollection<Action<()>> =
        ActionCollection::from_vec(vec![Action::IncrementHeight]);

    // Act
    let (mutations, outputs, executions) = partitioner.partition(&actions);

    // Assert
    assert_eq!(mutations.into_inner().len(), 1);
    assert_eq!(outputs.into_inner().len(), 0);
    assert_eq!(executions.into_inner().len(), 0);
}

#[test]
fn partition_output_action_goes_to_outputs_only() {
    // Arrange
    let partitioner = TypeBasedPartitioner::new();
    let actions: ActionCollection<Action<()>> = ActionCollection::from_vec(vec![Action::Log {
        message: "test".to_string(),
    }]);

    // Act
    let (mutations, outputs, executions) = partitioner.partition(&actions);

    // Assert
    assert_eq!(mutations.into_inner().len(), 0);
    assert_eq!(outputs.into_inner().len(), 1);
    assert_eq!(executions.into_inner().len(), 0);
}

#[test]
fn partition_mixed_actions_split_correctly() {
    // Arrange
    let partitioner = TypeBasedPartitioner::new();
    let actions: ActionCollection<Action<()>> = ActionCollection::from_vec(vec![
        Action::IncrementHeight,
        Action::Log {
            message: "a".to_string(),
        },
        Action::IncrementHeight,
        Action::Log {
            message: "b".to_string(),
        },
        Action::ExecuteBlock {
            block: Block::new(0, 0, vec![], [0u8; 32]),
        },
    ]);

    // Act
    let (mutations, outputs, executions) = partitioner.partition(&actions);

    // Assert
    assert_eq!(mutations.into_inner().len(), 2);
    assert_eq!(outputs.into_inner().len(), 2);
    assert_eq!(executions.into_inner().len(), 1);
}
