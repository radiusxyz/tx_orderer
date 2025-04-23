use std::collections::BTreeMap;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCluster {
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,
    pub cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterResponse {
    pub cluster_info: BTreeMap<u64, Cluster>,
}

impl RpcParameter<AppState> for GetCluster {
    type Response = GetClusterResponse;

    fn method() -> &'static str {
        "get_cluster"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let cluster_metadata = ClusterMetadata::get_or(
            self.platform,
            self.liveness_service_provider,
            &self.cluster_id,
            ClusterMetadata::default,
        )?;

        let liveness_service_manager_client = context
            .get_liveness_service_manager_client::<
                liveness_service_manager::radius::LivenessServiceManagerClient,
            >(self.platform, self.liveness_service_provider)
            .await?;

        let mut block_margin: u64 = liveness_service_manager_client
            .publisher()
            .get_block_margin()
            .await
            .expect("Failed to get block margin")
            .try_into()
            .expect("Failed to convert block margin");

        let mut cluster_info = BTreeMap::new();
        while block_margin > 0 {
            let platform_block_height = cluster_metadata.platform_block_height - block_margin;
            let cluster = Cluster::get(
                self.platform,
                self.liveness_service_provider,
                &self.cluster_id,
                platform_block_height,
            )?;

            cluster_info.insert(platform_block_height, cluster);
            block_margin -= 1;
        }

        Ok(GetClusterResponse { cluster_info })
    }
}
