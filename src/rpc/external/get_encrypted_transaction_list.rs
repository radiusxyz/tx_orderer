use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionList {
    pub rollup_id: String,
    pub rollup_block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionListResponse {
    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
}

impl RpcParameter<AppState> for GetEncryptedTransactionList {
    type Response = GetEncryptedTransactionListResponse;

    fn method() -> &'static str {
        "get_encrypted_transaction_list"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let block = Block::get(&self.rollup_id, self.rollup_block_height)?;

        Ok(GetEncryptedTransactionListResponse {
            encrypted_transaction_list: block.encrypted_transaction_list,
        })
    }
}
