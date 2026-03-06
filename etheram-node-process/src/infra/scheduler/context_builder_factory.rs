// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::context::context_builder::ContextBuilder;
use etheram_node::implementations::eager_context_builder::EagerContextBuilder;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;

pub fn build_context_builder() -> Result<Box<dyn ContextBuilder<IbftMessage>>, String> {
    Ok(Box::new(EagerContextBuilder::new()))
}
