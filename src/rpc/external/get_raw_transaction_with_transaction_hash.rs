use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

impl GetRawTransactionWithTransactionHash {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_with_transaction_hash";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<RawTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let raw_transaction = RawTransactionModel::get_with_transaction_hash(
            &parameter.rollup_id,
            &parameter.transaction_hash,
        )?;

        Ok(raw_transaction)
    }
}
