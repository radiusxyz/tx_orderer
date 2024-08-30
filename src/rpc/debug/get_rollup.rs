use crate::{models::RollupModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollup {
    rollup_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupResponse {
    rollup: Rollup,
    sequencing_info_key: SequencingInfoKey,
    cluster_id: ClusterId,
}

impl GetRollup {
    pub const METHOD_NAME: &'static str = "get_rollup";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRollupResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup_model = RollupModel::get(&parameter.rollup_id)?;

        let rollup = rollup_model.rollup().clone();

        let sequencing_info_key = rollup_model.sequencing_info_key().clone();

        let cluster_id = rollup_model.cluster_id().clone();

        Ok(GetRollupResponse {
            rollup,
            sequencing_info_key,
            cluster_id,
        })
    }
}
