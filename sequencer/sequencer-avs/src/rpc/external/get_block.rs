use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for GetBlock {
    type Response = Block;

    fn method_name() -> &'static str {
        stringify!(GetBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        let block = Block::get(self.rollup_block_number)?;
        Ok(block)
    }
}
