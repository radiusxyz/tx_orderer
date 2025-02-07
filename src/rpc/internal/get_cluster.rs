use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCluster {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub platform_block_height: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterResponse {
    pub cluster: Cluster,
    pub latest_cluster_block_height: u64,
}

impl RpcParameter<AppState> for GetCluster {
    type Response = GetClusterResponse;

    fn method() -> &'static str {
        "get_cluster"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let platform_block_height = if self.platform_block_height.is_none() {
            LatestClusterBlockHeight::get_or(
                self.platform,
                self.service_provider,
                &self.cluster_id,
                LatestClusterBlockHeight::default,
            )?
            .get_block_height()
        } else {
            self.platform_block_height.unwrap()
        };

        let cluster = Cluster::get(
            self.platform,
            self.service_provider,
            &self.cluster_id,
            platform_block_height,
        )?;

        Ok(GetClusterResponse {
            cluster,
            latest_cluster_block_height: platform_block_height,
        })
    }
}
