use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

impl GetEncryptedTransactionWithTransactionHash {
    pub const METHOD_NAME: &'static str = "get_encrypted_transaction_with_transaction_hash";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<EncryptedTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let encrypted_transaction = EncryptedTransactionModel::get_with_transaction_hash(
            &parameter.rollup_id,
            &parameter.transaction_hash,
        )?;

        Ok(encrypted_transaction)
    }
}
