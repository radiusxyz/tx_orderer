use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCluster {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterResponse {
    pub cluster_info: Cluster,
}

impl GetCluster {
    pub const METHOD_NAME: &'static str = "get_cluster";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<GetClusterResponse, RpcError> {
        let parameter = parameter.parse::<GetCluster>()?;

        let liveness_client_info =
            SequencingInfoPayload::get(parameter.platform, parameter.service_provider)?;
        match liveness_client_info {
            SequencingInfoPayload::Ethereum(_) => {
                let liveness_client = context
                    .get_liveness_client::<liveness::radius::LivenessClient>(
                        parameter.platform,
                        parameter.service_provider,
                    )
                    .await?;
                let platform_block_height = liveness_client.publisher().get_block_number().await?;
                let cluster_info = Cluster::get(
                    parameter.platform,
                    parameter.service_provider,
                    &parameter.cluster_id,
                    platform_block_height,
                )?;

                Ok(GetClusterResponse { cluster_info })
            }
            SequencingInfoPayload::Local(_) => {
                unimplemented!("Local liveness is unimplemented.");
            }
        }
    }
}
