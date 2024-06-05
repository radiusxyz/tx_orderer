use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for SyncBuildBlock {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SyncBuildBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
        Ok(())
    }
}
