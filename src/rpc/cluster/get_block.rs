use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_id: String,
    pub rollup_block_height: u64,
}

impl GetBlock {
    pub const METHOD_NAME: &'static str = "get_block";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<Block, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        BlockModel::get(&parameter.rollup_id, parameter.rollup_block_height)
            .map_err(|error| error.into())
    }
}
