use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionList {
    pub rollup_id: RollupId,
    pub batch_number: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionListResponse {
    pub raw_transaction_list: Vec<String>,
}

impl RpcParameter<AppState> for GetRawTransactionList {
    type Response = GetRawTransactionListResponse;

    fn method() -> &'static str {
        "get_raw_transaction_list"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let batch = Batch::get(&self.rollup_id, self.batch_number)?;

        let raw_transaction_list: Vec<String> = batch
            .raw_transaction_list
            .into_iter()
            .map(|transaction| match transaction {
                RawTransaction::Eth(EthRawTransaction(data)) => data,
                RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
            })
            .collect();

        Ok(GetRawTransactionListResponse {
            raw_transaction_list,
        })
    }
}
