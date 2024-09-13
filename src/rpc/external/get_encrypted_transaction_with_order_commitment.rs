use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithOrderCommitment {
    rollup_id: String,
    rollup_block_height: u64,
    transaction_order: u64,
}

impl GetEncryptedTransactionWithOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_encrypted_transaction_with_order_commitment";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<EncryptedTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let encrypted_transaction = EncryptedTransactionModel::get(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
        )?;

        Ok(encrypted_transaction)
    }
}
