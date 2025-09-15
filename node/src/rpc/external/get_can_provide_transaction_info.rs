use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCanProvideTransactionInfo {
    pub rollup_id: RollupId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCanProvideTransactionInfoResponse {
    pub can_provide_transaction_info: CanProvideTransactionInfo,
}

impl RpcParameter<AppState> for GetCanProvideTransactionInfo {
    type Response = GetCanProvideTransactionInfoResponse;

    fn method() -> &'static str {
        "get_can_provide_transaction_info"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let can_provide_transaction_info = CanProvideTransactionInfo::get(&self.rollup_id)?;

        Ok(GetCanProvideTransactionInfoResponse {
            can_provide_transaction_info,
        })
    }
}
