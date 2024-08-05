use crate::{models::ClusterIdListModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddCluster {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,

    cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddClusterResponse {
    success: bool,
}

impl AddCluster {
    pub const METHOD_NAME: &'static str = "add_cluster";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<AddClusterResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // TODO: get operator information
        let mut cluster_id_list_model = ClusterIdListModel::entry(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )?;

        cluster_id_list_model.push(parameter.cluster_id.clone());
        cluster_id_list_model.update()?;

        Ok(AddClusterResponse { success: true })
    }
}
