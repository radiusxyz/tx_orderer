use crate::{
    models::{ClusterIdListModel, RollupIdListModel, RollupModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateRollup {
    // to update data
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,
    cluster_id: ClusterId,

    rollup_id: RollupId,
    rollup_type: RollupType,

    // changable fields
    rollup_rpc_url: IpAddress,
    rollup_websocket_url: IpAddress,

    bundler_contract_address: Option<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateRollupResponse {
    success: bool,
}

impl UpdateRollup {
    pub const METHOD_NAME: &'static str = "update_rollup";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<UpdateRollupResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup_id = parameter.rollup_id.clone();

        let cluster_id_list_model = match ClusterIdListModel::get(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        ) {
            Ok(cluster_id_list_model) => cluster_id_list_model,
            Err(err) => {
                if err.is_none_type() {
                    tracing::error!("Cluster is not registered");
                    return Ok(UpdateRollupResponse { success: false });
                } else {
                    return Err(err.into());
                }
            }
        };

        if !cluster_id_list_model.is_added_cluster_id(&parameter.cluster_id) {
            return Ok(UpdateRollupResponse { success: false });
        }

        let rollup_id_list_model = match RollupIdListModel::get() {
            Ok(rollup_id_list_model) => rollup_id_list_model,
            Err(err) => {
                if err.is_none_type() {
                    tracing::error!("Rollup is not registered");
                    return Ok(UpdateRollupResponse { success: false });
                } else {
                    return Err(err.into());
                }
            }
        };

        if !rollup_id_list_model.is_exist_rollup_id(&rollup_id) {
            return Ok(UpdateRollupResponse { success: false });
        }

        // update rollup
        let mut rollup_model = RollupModel::get_mut(&rollup_id)?;

        let rollup = Rollup::new(
            rollup_model.rollup().rollup_id().clone(),
            rollup_model.rollup().rollup_type().clone(),
            parameter.rollup_rpc_url,
            parameter.rollup_websocket_url,
            parameter.bundler_contract_address,
        );

        rollup_model.update_rollup(rollup);
        rollup_model.update()?;

        Ok(UpdateRollupResponse { success: true })
    }
}
