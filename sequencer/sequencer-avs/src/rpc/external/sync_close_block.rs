use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncCloseBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for SyncCloseBlock {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SyncCloseBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
        Ok(())
    }
}
