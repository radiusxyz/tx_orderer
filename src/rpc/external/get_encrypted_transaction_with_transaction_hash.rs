use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

impl RpcParameter<AppState> for GetEncryptedTransactionWithTransactionHash {
    type Response = EncryptedTransaction;

    fn method() -> &'static str {
        "get_encrypted_transaction_with_transaction_hash"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let encrypted_transaction = EncryptedTransactionModel::get_with_transaction_hash(
            &self.rollup_id,
            &self.transaction_hash,
        )?;

        Ok(encrypted_transaction)
    }
}
