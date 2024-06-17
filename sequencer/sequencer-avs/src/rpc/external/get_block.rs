use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_block_number: RollupBlockNumber,
}

impl GetBlock {
    pub const METHOD_NAME: &'static str = stringify!(GetBlock);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<SsalClient>,
    ) -> Result<RollupBlock, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let block = RollupBlock::get(parameter.rollup_block_number)?;
        Ok(block)
    }
}
