use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCluster {
    platform: Platform,
    service_provider: ServiceProvider,

    cluster_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterResponse {
    cluster_info: Cluster,
}

impl GetCluster {
    pub const METHOD_NAME: &'static str = "get_cluster";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<GetClusterResponse, RpcError> {
        let parameter = parameter.parse::<GetCluster>()?;

        match context.get_liveness_client(parameter.platform, parameter.service_provider) {
            Some(liveness_client) => {
                let block_number = liveness_client.publisher().get_block_number().await?;

                let cluster_info = ClusterModel::get(&parameter.cluster_id, block_number)?;

                Ok(GetClusterResponse { cluster_info })
            }
            None => Err(Error::NotFoundCluster.into()),
        }
    }
}
