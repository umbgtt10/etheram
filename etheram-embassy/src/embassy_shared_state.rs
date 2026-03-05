// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::sync::Arc;
use core::cell::RefCell;
use critical_section::Mutex;
use etheram_core::node_common::shared_state::SharedState;

pub struct EmbassySharedState<T> {
    state: Arc<Mutex<RefCell<T>>>,
}

impl<T> Clone for EmbassySharedState<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<T> EmbassySharedState<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: Arc::new(Mutex::new(RefCell::new(value))),
        }
    }
}

impl<T> SharedState<T> for EmbassySharedState<T> {
    type Lock = Mutex<RefCell<T>>;

    fn with<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        critical_section::with(|cs| {
            let state = self.state.borrow(cs).borrow();
            operation(&state)
        })
    }

    fn with_mut<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        critical_section::with(|cs| {
            let mut state = self.state.borrow(cs).borrow_mut();
            operation(&mut state)
        })
    }
}
