// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_proto::wire::BlockMessage;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetBalanceReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetHeightReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::SubmitTransactionReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::TransactionMessage;
use crate::infra::external_interface::grpc_external_interface_proto::wire::TransactionRejectionReason;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason as NodeTransactionRejectionReason;

pub struct GrpcExternalInterfaceCodec;

impl GrpcExternalInterfaceCodec {
    pub fn decode_address(bytes: &[u8]) -> Result<Address, String> {
        bytes
            .try_into()
            .map_err(|_| format!("invalid address length: {}", bytes.len()))
    }

    pub fn decode_transaction(message: &TransactionMessage) -> Result<Transaction, String> {
        Ok(Transaction::new(
            Self::decode_address(&message.from)?,
            Self::decode_address(&message.to)?,
            message.value.parse::<u128>().map_err(|error| {
                format!("invalid transaction value '{}': {}", message.value, error)
            })?,
            message.gas_limit,
            message.gas_price,
            message.nonce,
            message.data.clone(),
        ))
    }

    pub fn encode_balance_response(response: &ClientResponse) -> Result<GetBalanceReply, String> {
        match response {
            ClientResponse::Balance { balance, height } => Ok(GetBalanceReply {
                balance: balance.to_string(),
                height: *height,
            }),
            other => Err(format!(
                "unexpected client response for balance request: {:?}",
                other
            )),
        }
    }

    pub fn encode_block(block: &Block) -> BlockMessage {
        BlockMessage {
            height: block.height,
            proposer: block.proposer,
            transactions: block
                .transactions
                .iter()
                .map(Self::encode_transaction)
                .collect(),
            state_root: block.state_root.to_vec(),
            post_state_root: block.post_state_root.to_vec(),
            receipts_root: block.receipts_root.to_vec(),
            gas_limit: block.gas_limit,
        }
    }

    pub fn encode_height_response(response: &ClientResponse) -> Result<GetHeightReply, String> {
        match response {
            ClientResponse::Height(height) => Ok(GetHeightReply { height: *height }),
            other => Err(format!(
                "unexpected client response for height request: {:?}",
                other
            )),
        }
    }

    pub fn encode_submit_transaction_response(
        response: &ClientResponse,
    ) -> Result<SubmitTransactionReply, String> {
        match response {
            ClientResponse::TransactionAccepted => Ok(SubmitTransactionReply {
                accepted: true,
                rejection_reason: TransactionRejectionReason::Unspecified as i32,
            }),
            ClientResponse::TransactionRejected { reason } => Ok(SubmitTransactionReply {
                accepted: false,
                rejection_reason: Self::encode_transaction_rejection_reason(reason) as i32,
            }),
            other => Err(format!(
                "unexpected client response for submit transaction request: {:?}",
                other
            )),
        }
    }

    pub fn encode_transaction(transaction: &Transaction) -> TransactionMessage {
        TransactionMessage {
            from: transaction.from.to_vec(),
            to: transaction.to.to_vec(),
            value: transaction.value.to_string(),
            gas_limit: transaction.gas_limit,
            gas_price: transaction.gas_price,
            nonce: transaction.nonce,
            data: transaction.data.clone(),
        }
    }

    fn encode_transaction_rejection_reason(
        reason: &NodeTransactionRejectionReason,
    ) -> TransactionRejectionReason {
        match reason {
            NodeTransactionRejectionReason::GasLimitExceeded => {
                TransactionRejectionReason::GasLimitExceeded
            }
            NodeTransactionRejectionReason::InsufficientBalance => {
                TransactionRejectionReason::InsufficientBalance
            }
            NodeTransactionRejectionReason::InvalidNonce => {
                TransactionRejectionReason::InvalidNonce
            }
            NodeTransactionRejectionReason::ZeroGasPrice => {
                TransactionRejectionReason::ZeroGasPrice
            }
        }
    }
}
