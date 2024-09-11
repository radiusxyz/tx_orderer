use crate::{models::ClusterIdListModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdList {
    platform: Platform,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterIdListResponse {
    cluster_id_list: ClusterIdList,
}

impl GetClusterIdList {
    pub const METHOD_NAME: &'static str = "get_cluster_id_list";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetClusterIdListResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster_id_list = ClusterIdListModel::get(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )?
        .cluster_id_list();

        Ok(GetClusterIdListResponse { cluster_id_list })
    }
}
