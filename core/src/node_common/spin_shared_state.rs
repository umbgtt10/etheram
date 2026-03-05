// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use crate::node_common::shared_state::SharedState;
use alloc::sync::Arc;
use spin::Mutex;

pub struct SpinSharedState<T> {
    state: Arc<Mutex<T>>,
}

impl<T> Clone for SpinSharedState<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<T> SpinSharedState<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: Arc::new(Mutex::new(value)),
        }
    }
}

impl<T> SharedState<T> for SpinSharedState<T> {
    type Lock = Mutex<T>;

    fn with<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.state.lock();
        operation(&guard)
    }

    fn with_mut<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.state.lock();
        operation(&mut guard)
    }
}
