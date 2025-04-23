use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithOrderCommitment {
    pub rollup_id: RollupId,
    pub batch_number: u64,
    pub transaction_order: u64,
}

impl RpcParameter<AppState> for GetEncryptedTransactionWithOrderCommitment {
    type Response = EncryptedTransaction;

    fn method() -> &'static str {
        "get_encrypted_transaction_with_order_commitment"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let encrypted_transaction = EncryptedTransactionModel::get(
            &self.rollup_id,
            self.batch_number,
            self.transaction_order,
        )?;

        Ok(encrypted_transaction)
    }
}
