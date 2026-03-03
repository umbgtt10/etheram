// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::ibft::ibft_message::IbftMessage;
use crate::implementations::ibft::ibft_protocol::IbftProtocol;
use crate::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use crate::implementations::no_op_protocol::NoOpProtocol;
use crate::variants::ProtocolVariant;
use alloc::boxed::Box;
use etheram::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram::execution::execution_engine::BoxedExecutionEngine;

pub struct ProtocolBuilder<M> {
    protocol: Option<BoxedProtocol<M>>,
    execution_engine: Option<BoxedExecutionEngine>,
}

impl<M> ProtocolBuilder<M> {
    pub fn new() -> Self {
        Self {
            protocol: None,
            execution_engine: None,
        }
    }

    pub fn with_protocol(mut self, protocol: BoxedProtocol<M>) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn build(self) -> Result<BoxedProtocol<M>, BuildError> {
        self.protocol
            .ok_or(BuildError::MissingComponent("protocol"))
    }
}

impl ProtocolBuilder<()> {
    pub fn with_variant(mut self, variant: ProtocolVariant<()>) -> Self {
        let protocol: BoxedProtocol<()> = match variant {
            ProtocolVariant::Ibft { .. } => return self,
            ProtocolVariant::NoOp => Box::new(NoOpProtocol::<()>::new()),
            ProtocolVariant::Custom(custom) => custom,
        };
        self.protocol = Some(protocol);
        self
    }
}

impl ProtocolBuilder<IbftMessage> {
    pub fn with_execution_engine(mut self, engine: BoxedExecutionEngine) -> Self {
        self.execution_engine = Some(engine);
        self
    }

    pub fn with_variant(mut self, variant: ProtocolVariant<IbftMessage>) -> Self {
        let protocol: BoxedProtocol<IbftMessage> = match variant {
            ProtocolVariant::Ibft { validators } => {
                let ibft = IbftProtocol::new(validators, Box::new(MockSignatureScheme::new(0)));
                let ibft = match self.execution_engine.take() {
                    Some(engine) => ibft.with_execution_engine(engine),
                    None => ibft,
                };
                Box::new(ibft)
            }
            ProtocolVariant::NoOp => Box::new(NoOpProtocol::<IbftMessage>::new()),
            ProtocolVariant::Custom(custom) => custom,
        };
        self.protocol = Some(protocol);
        self
    }
}

impl Default for ProtocolBuilder<()> {
    fn default() -> Self {
        Self {
            protocol: Some(Box::new(NoOpProtocol::<()>::new())),
            execution_engine: None,
        }
    }
}
