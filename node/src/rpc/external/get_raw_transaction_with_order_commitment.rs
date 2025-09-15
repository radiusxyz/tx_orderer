use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitment {
    pub rollup_id: RollupId,
    pub batch_number: u64,
    pub transaction_order: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitmentResponse {
    pub raw_transaction: RawTransaction,
    pub is_direct_sent: bool,
}

impl RpcParameter<AppState> for GetRawTransactionWithOrderCommitment {
    type Response = GetRawTransactionWithOrderCommitmentResponse;

    fn method() -> &'static str {
        "get_raw_transaction_with_order_commitment"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let (raw_transaction, is_direct_sent) =
            RawTransactionModel::get(&self.rollup_id, self.batch_number, self.transaction_order)?;

        Ok(GetRawTransactionWithOrderCommitmentResponse {
            raw_transaction,
            is_direct_sent,
        })
    }
}
