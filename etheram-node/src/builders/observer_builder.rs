// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_observer::NoOpObserver;
use crate::observer::Observer;
use crate::variants::ObserverVariant;
use alloc::boxed::Box;

pub struct ObserverBuilder {
    observer: Option<Box<dyn Observer>>,
}

impl ObserverBuilder {
    pub fn new() -> Self {
        Self { observer: None }
    }

    pub fn with_variant(mut self, variant: ObserverVariant) -> Self {
        let observer = match variant {
            ObserverVariant::NoOp => Box::new(NoOpObserver) as Box<dyn Observer>,
            ObserverVariant::Custom(custom) => custom,
        };
        self.observer = Some(observer);
        self
    }

    pub fn build(self) -> Result<Box<dyn Observer>, BuildError> {
        self.observer
            .ok_or(BuildError::MissingComponent("observer"))
    }
}

impl Default for ObserverBuilder {
    fn default() -> Self {
        Self::new().with_variant(ObserverVariant::NoOp)
    }
}
