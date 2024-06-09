use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub transaction: Transaction,
    pub order_commitment: OrderCommitment,
}

#[async_trait]
impl RpcMethod for SyncTransaction {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SyncTransaction)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        self.transaction.put(
            self.order_commitment.rollup_block_number(),
            self.order_commitment.transaction_order(),
        )?;
        Ok(())
    }
}
