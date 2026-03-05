// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::implementations::shared_state::SharedState;
use std::sync::Arc;
use std::sync::Mutex;

pub struct StdSharedState<T> {
    state: Arc<Mutex<T>>,
}

impl<T> Clone for StdSharedState<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<T> StdSharedState<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: Arc::new(Mutex::new(value)),
        }
    }
}

impl<T> SharedState<T> for StdSharedState<T> {
    type Lock = Mutex<T>;

    fn with<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.state.lock().unwrap();
        operation(&guard)
    }

    fn with_mut<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.state.lock().unwrap();
        operation(&mut guard)
    }
}
