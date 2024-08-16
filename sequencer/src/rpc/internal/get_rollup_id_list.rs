use crate::{models::RollupIdListModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupIdList {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupIdListResponse {
    rollup_id_list: RollupIdList,
}

impl GetRollupIdList {
    pub const METHOD_NAME: &'static str = "get_rollup_list";

    pub async fn handler(
        _parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRollupIdListResponse, RpcError> {
        let rollup_id_list = RollupIdListModel::get()?.rollup_id_list().clone();

        Ok(GetRollupIdListResponse { rollup_id_list })
    }
}
