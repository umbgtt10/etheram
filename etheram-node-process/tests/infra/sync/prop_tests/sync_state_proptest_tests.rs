// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node_process::infra::sync::sync_state::SyncState;
use proptest::prelude::*;
use std::collections::BTreeMap;

fn arb_peer_id() -> BoxedStrategy<u64> {
    (1u64..=10u64).boxed()
}

fn arb_height() -> BoxedStrategy<u64> {
    (0u64..=1000u64).boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn observe_status_highest_peer_height_returns_max(
        heights in proptest::collection::vec((arb_peer_id(), arb_height()), 1..=5),
    ) {
        // Arrange
        let mut state = SyncState::new();
        let mut last_per_peer = BTreeMap::new();

        // Act
        for (peer, height) in &heights {
            state.observe_status(*peer, *height);
            last_per_peer.insert(*peer, *height);
        }

        // Assert
        let expected_max = last_per_peer.values().copied().max();
        prop_assert_eq!(state.highest_peer_height(), expected_max);
    }

    #[test]
    fn lag_distance_correct_when_peers_ahead(
        local in 0u64..500u64,
        remote_offset in 1u64..500u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        let remote = local + remote_offset;
        state.observe_status(1, remote);

        // Act
        let lag = state.lag_distance(local);

        // Assert
        prop_assert_eq!(lag, Some(remote_offset));
    }

    #[test]
    fn lag_distance_none_when_local_at_or_ahead(
        local in 0u64..1000u64,
        remote in 0u64..1000u64,
    ) {
        // Arrange
        prop_assume!(remote <= local);
        let mut state = SyncState::new();
        state.observe_status(1, remote);

        // Act
        let lag = state.lag_distance(local);

        // Assert
        prop_assert!(lag.is_none());
    }

    #[test]
    fn next_request_picks_peer_ahead_of_local(
        local in 0u64..100u64,
        peer_heights in proptest::collection::vec(
            (arb_peer_id(), arb_height()),
            1..=5,
        ),
    ) {
        // Arrange
        let mut state = SyncState::new();
        let mut last_per_peer = BTreeMap::new();
        for (peer, height) in &peer_heights {
            state.observe_status(*peer, *height);
            last_per_peer.insert(*peer, *height);
        }
        let has_peer_ahead = last_per_peer.values().any(|h| *h > local);

        // Act
        let request = state.next_request(local, 10);

        // Assert
        if has_peer_ahead {
            prop_assert!(request.is_some());
            let (_, start_height, max_blocks) = request.unwrap();
            prop_assert_eq!(start_height, local);
            prop_assert_eq!(max_blocks, 10);
        } else {
            prop_assert!(request.is_none());
        }
    }

    #[test]
    fn next_request_returns_none_when_in_flight_exists(
        local in 0u64..100u64,
        remote in 101u64..200u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        state.observe_status(1, remote);
        state.next_request(local, 10);

        // Act
        let second = state.next_request(local, 10);

        // Assert
        prop_assert!(second.is_none());
    }

    #[test]
    fn complete_in_flight_clears_request(
        local in 0u64..100u64,
        remote in 101u64..200u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        state.observe_status(1, remote);
        let (peer, start, _) = state.next_request(local, 10).unwrap();

        // Act
        let completed = state.complete_in_flight_request(peer, start);

        // Assert
        prop_assert!(completed);
        let next = state.next_request(local, 10);
        prop_assert!(next.is_some());
    }

    #[test]
    fn fail_in_flight_allows_different_peer(
        local in 0u64..100u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        state.observe_status(1, local + 50);
        state.observe_status(2, local + 50);
        let (peer, start, _) = state.next_request(local, 10).unwrap();

        // Act
        let failed = state.fail_in_flight_request(peer, start);

        // Assert
        prop_assert!(failed);
        let next = state.next_request(local, 10);
        prop_assert!(next.is_some());
        let (next_peer, _, _) = next.unwrap();
        prop_assert_ne!(next_peer, peer);
    }

    #[test]
    fn complete_mismatched_peer_returns_false(
        local in 0u64..100u64,
        remote in 101u64..200u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        state.observe_status(1, remote);
        state.next_request(local, 10);

        // Act
        let result = state.complete_in_flight_request(999, local);

        // Assert
        prop_assert!(!result);
    }

    #[test]
    fn complete_mismatched_height_returns_false(
        local in 0u64..100u64,
        remote in 101u64..200u64,
    ) {
        // Arrange
        let mut state = SyncState::new();
        state.observe_status(1, remote);
        state.next_request(local, 10);

        // Act
        let result = state.complete_in_flight_request(1, local + 1);

        // Assert
        prop_assert!(!result);
    }
}
