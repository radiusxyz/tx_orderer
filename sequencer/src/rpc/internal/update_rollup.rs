use crate::{
    models::{ClusterIdListModel, RollupIdListModel, RollupMetadataModel, RollupModel},
    rpc::prelude::*,
    state::RollupState,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateRollup {
    previous_rollup_id: RollupId,

    // to update data
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,
    cluster_id: ClusterId,

    rollup_id: RollupId,
    rollup_type: RollupType,

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
        context: Arc<AppState>,
    ) -> Result<UpdateRollupResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let previous_rollup_id = parameter.previous_rollup_id.clone();

        let cluster_id_list_model = ClusterIdListModel::get(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )?;

        let is_added_cluster = cluster_id_list_model
            .cluster_id_list
            .contains(&parameter.cluster_id);

        if !is_added_cluster {
            return Ok(UpdateRollupResponse { success: false });
        }

        let mut rollup_id_list_model = RollupIdListModel::get_mut_or_init()?;

        if !rollup_id_list_model.is_exist_rollup_id(&previous_rollup_id) {
            return Ok(UpdateRollupResponse { success: false });
        }

        if previous_rollup_id != parameter.rollup_id {
            let mut rollup_id_list = rollup_id_list_model.rollup_id_list().clone();
            rollup_id_list.retain(|rollup_id| rollup_id != &previous_rollup_id);

            rollup_id_list_model.update_rollup_id_list(rollup_id_list);
            rollup_id_list_model.update()?;
        }

        // update rollup
        let mut rollup_model = RollupModel::get_mut(&previous_rollup_id)?;

        let rollup = Rollup::new(
            parameter.rollup_id.clone(),
            parameter.rollup_type,
            parameter.rollup_rpc_url,
            parameter.rollup_websocket_url,
            parameter.bundler_contract_address,
        );

        let sequencing_info_key = SequencingInfoKey::new(
            parameter.platform,
            parameter.sequencing_function_type,
            parameter.service_type,
        );

        let new_rollup_model =
            RollupModel::new(rollup, sequencing_info_key, parameter.cluster_id.clone());
        *rollup_model = new_rollup_model;
        rollup_model.update()?;

        // update rollup metadata
        let mut rollup_metadata_model: Lock<'_, RollupMetadataModel> =
            RollupMetadataModel::get_mut(&previous_rollup_id)?;

        let rollup_metadata = rollup_metadata_model.rollup_metadata().clone();

        let block_height = rollup_metadata.block_height();

        let new_rollup_metadata_model =
            RollupMetadataModel::new(parameter.rollup_id.clone(), rollup_metadata);

        *rollup_metadata_model = new_rollup_metadata_model;
        rollup_metadata_model.update()?;

        context.set_rollup_state(parameter.rollup_id.clone(), RollupState::new(block_height));
        context.set_cluster_id(parameter.rollup_id, parameter.cluster_id);

        Ok(UpdateRollupResponse { success: true })
    }
}
