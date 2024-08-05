use sequencer::{
    models::{ClusterIdListModel, ClusterModel, LivenessClusterModel, ValidationClusterModel},
    types::{PlatForm, SequencingFunctionType, ServiceType},
};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterList {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterListResponse {
    cluster_list: Vec<ClusterModel>,
}

impl GetClusterList {
    pub const METHOD_NAME: &'static str = "get_cluster_list";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetClusterListResponse, RpcError> {
        let parameter = parameter.parse::<GetClusterList>()?;

        let cluster_id_list_model = ClusterIdListModel::get(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )
        .unwrap_or_default();

        let cluster_list: Vec<ClusterModel> = cluster_id_list_model
            .cluster_id_list
            .iter()
            .map(|cluster_id| match parameter.sequencing_function_type {
                SequencingFunctionType::Liveness => ClusterModel::Liveness(
                    LivenessClusterModel::get(
                        &parameter.platform,
                        &parameter.service_type,
                        &cluster_id,
                    )
                    .unwrap(),
                ),
                SequencingFunctionType::Validation => ClusterModel::Validation(
                    ValidationClusterModel::get(
                        &parameter.platform,
                        &parameter.service_type,
                        &cluster_id,
                    )
                    .unwrap(),
                ),
            })
            .collect();

        Ok(GetClusterListResponse { cluster_list })
    }
}
