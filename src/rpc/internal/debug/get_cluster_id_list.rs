use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdList {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdListResponse {
    cluster_id_list: ClusterIdList,
}

impl GetClusterIdList {
    pub const METHOD_NAME: &'static str = "get_cluster_id_list";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetClusterIdListResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster_id_list = ClusterIdList::get(parameter.platform, parameter.service_provider)?;

        Ok(GetClusterIdListResponse { cluster_id_list })
    }
}
