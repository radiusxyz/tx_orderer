use crate::{models::RollupBlockModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_id: ClusterId,
    pub rollup_block_height: BlockHeight,
}

impl GetBlock {
    pub const METHOD_NAME: &'static str = stringify!(GetBlock);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<RollupBlockModel, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        RollupBlockModel::get(&parameter.rollup_id, &parameter.rollup_block_height)
            .map_err(|error| error.into())
    }
}
