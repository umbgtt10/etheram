// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait SharedState<T>: Clone {
    type Lock;

    fn with<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&T) -> R;

    fn with_mut<R, F>(&self, operation: F) -> R
    where
        F: FnOnce(&mut T) -> R;
}
