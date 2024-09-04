use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCluster {
    platform: Platform,
    service_provider: ServiceProvider,
    cluster_id: String,
}

impl JoinCluster {
    pub const METHOD_NAME: &'static str = "join_cluster";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut cluster_id_list =
            ClusterIdListModel::get_mut(parameter.platform, parameter.service_provider)?;
        cluster_id_list.insert(&parameter.cluster_id);
        cluster_id_list.update()?;

        Ok(())
    }
}
