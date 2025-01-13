use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdList {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdListResponse {
    pub cluster_id_list: ClusterIdList,
}

impl RpcParameter<AppState> for GetClusterIdList {
    type Response = GetClusterIdListResponse;

    fn method() -> &'static str {
        "get_cluster_id_list"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let cluster_id_list = ClusterIdList::get(self.platform, self.service_provider)?;

        Ok(GetClusterIdListResponse { cluster_id_list })
    }
}
