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
}

impl RpcParameter<AppState> for GetCluster {
    type Response = GetClusterResponse;

    fn method() -> &'static str {
        "get_cluster"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let liveness_client_info =
            SequencingInfoPayload::get(self.platform, self.service_provider)?;
        match liveness_client_info {
            SequencingInfoPayload::Ethereum(_) => {
                let liveness_client = context
                    .get_liveness_client::<liveness::radius::LivenessClient>(
                        self.platform,
                        self.service_provider,
                    )
                    .await?;

                let platform_block_height =
                    if let Some(platform_block_height) = self.platform_block_height {
                        platform_block_height
                    } else {
                        let block_height = liveness_client
                            .publisher()
                            .get_block_number()
                            .await
                            .map_err(|error| Error::Internal(error.into()))?;

                        block_height
                    };

                let cluster = context
                    .get_cluster(
                        self.platform,
                        self.service_provider,
                        &self.cluster_id,
                        platform_block_height,
                    )
                    .await?;

                Ok(GetClusterResponse { cluster })
            }
            SequencingInfoPayload::Local(_) => {
                unimplemented!("Local liveness is unimplemented.");
            }
        }
    }
}
