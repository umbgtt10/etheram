// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_execution_engine::NoOpExecutionEngine;
use crate::implementations::tiny_evm_engine::TinyEvmEngine;
use crate::implementations::value_transfer_engine::ValueTransferEngine;
use crate::variants::ExecutionEngineVariant;
use alloc::boxed::Box;
use etheram_node::execution::execution_engine::BoxedExecutionEngine;

pub struct ExecutionEngineBuilder {
    execution_engine: Option<BoxedExecutionEngine>,
}

impl ExecutionEngineBuilder {
    pub fn new() -> Self {
        Self {
            execution_engine: None,
        }
    }

    pub fn with_variant(mut self, variant: ExecutionEngineVariant) -> Self {
        let execution_engine: BoxedExecutionEngine = match variant {
            ExecutionEngineVariant::NoOp => Box::new(NoOpExecutionEngine),
            ExecutionEngineVariant::TinyEvm => Box::new(TinyEvmEngine),
            ExecutionEngineVariant::ValueTransfer => Box::new(ValueTransferEngine),
            ExecutionEngineVariant::Custom(custom) => custom,
        };
        self.execution_engine = Some(execution_engine);
        self
    }

    pub fn build(self) -> Result<BoxedExecutionEngine, BuildError> {
        self.execution_engine
            .ok_or(BuildError::MissingComponent("execution_engine"))
    }
}

impl Default for ExecutionEngineBuilder {
    fn default() -> Self {
        Self {
            execution_engine: Some(Box::new(ValueTransferEngine)),
        }
    }
}
