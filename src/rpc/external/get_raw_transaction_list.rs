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

impl GetRawTransactionList {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_list";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRawTransactionListResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let block = Block::get(&parameter.rollup_id, parameter.rollup_block_height)?;

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
