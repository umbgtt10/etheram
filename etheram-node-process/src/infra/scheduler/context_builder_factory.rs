// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::context_builder_builder::ContextBuilderBuilder;
use etheram_node::context::context_builder::ContextBuilder;

pub fn build_context_builder() -> Result<Box<dyn ContextBuilder<()>>, String> {
    ContextBuilderBuilder::default()
        .build()
        .map_err(|error| format!("failed to build context builder: {error:?}"))
}
