use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionList {
    pub rollup_id: String,
    pub rollup_block_height: u64,
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
        let block = Block::get(&self.rollup_id, self.rollup_block_height)?;

        let raw_transaction_list: Vec<String> = block
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
