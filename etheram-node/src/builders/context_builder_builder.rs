// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::context::context_builder::ContextBuilder;
use crate::implementations::eager_context_builder::EagerContextBuilder;
use crate::variants::ContextBuilderVariant;
use alloc::boxed::Box;
use etheram_core::node_common::build_error::BuildError;

pub struct ContextBuilderBuilder {
    context_builder: Option<Box<dyn ContextBuilder<()>>>,
}

impl ContextBuilderBuilder {
    pub fn new() -> Self {
        Self {
            context_builder: None,
        }
    }
    pub fn with_variant(mut self, variant: ContextBuilderVariant) -> Self {
        let builder = match variant {
            ContextBuilderVariant::Eager => {
                Box::new(EagerContextBuilder::new()) as Box<dyn ContextBuilder<()>>
            }
            ContextBuilderVariant::Custom(custom) => custom,
        };
        self.context_builder = Some(builder);
        self
    }
    pub fn build(self) -> Result<Box<dyn ContextBuilder<()>>, BuildError> {
        self.context_builder
            .ok_or(BuildError::MissingComponent("context_builder"))
    }
}

impl Default for ContextBuilderBuilder {
    fn default() -> Self {
        Self::new().with_variant(ContextBuilderVariant::Eager)
    }
}
