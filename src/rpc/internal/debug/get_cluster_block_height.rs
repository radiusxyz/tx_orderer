use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterBlockHeight {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterBlockHeightResponse {
    cluster_block_height: u64,
}

impl GetClusterBlockHeight {
    pub const METHOD_NAME: &'static str = "get_cluster_block_height";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetClusterBlockHeightResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster_block_height = ClusterBlockHeight::get(
            parameter.platform,
            parameter.service_provider,
            &parameter.cluster_id,
        )?;

        Ok(GetClusterBlockHeightResponse {
            cluster_block_height: cluster_block_height.inner(),
        })
    }
}
