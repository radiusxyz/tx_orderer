use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithTransactionHashResponse {
    pub raw_transaction: RawTransaction,
    pub is_direct_sent: bool,
}

impl RpcParameter<AppState> for GetRawTransactionWithTransactionHash {
    type Response = GetRawTransactionWithTransactionHashResponse;

    fn method() -> &'static str {
        "get_raw_transaction_with_transaction_hash"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let (raw_transaction, is_direct_sent) = RawTransactionModel::get_with_transaction_hash(
            &self.rollup_id,
            &self.transaction_hash,
        )?;

        Ok(GetRawTransactionWithTransactionHashResponse {
            raw_transaction,
            is_direct_sent,
        })
    }
}
