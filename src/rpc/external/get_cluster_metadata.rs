use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterMetadata {
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,
    pub cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterMetadataResponse {
    pub cluster_metadata: ClusterMetadata,
}

impl RpcParameter<AppState> for GetClusterMetadata {
    type Response = GetClusterMetadataResponse;

    fn method() -> &'static str {
        "get_cluster_metadata"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let cluster_metadata = ClusterMetadata::get(
            self.platform,
            self.liveness_service_provider,
            &self.cluster_id,
        )?;

        Ok(GetClusterMetadataResponse { cluster_metadata })
    }
}
