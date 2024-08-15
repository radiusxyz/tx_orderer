use crate::{
    models::{ClusterIdListModel, RollupIdListModel, RollupMetadataModel, RollupModel},
    rpc::prelude::*,
    state::RollupState,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddRollup {
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
pub struct AddRollupResponse {
    success: bool,
}

impl AddRollup {
    pub const METHOD_NAME: &'static str = "add_rollup";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<AddRollupResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup_id = parameter.rollup_id.clone();

        let cluster_id_list_model = ClusterIdListModel::get(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )?;

        let is_added_cluster = cluster_id_list_model
            .cluster_id_list
            .contains(&parameter.cluster_id);

        if !is_added_cluster {
            return Ok(AddRollupResponse { success: false });
        }

        let mut rollup_id_list_model = RollupIdListModel::entry()?;

        if rollup_id_list_model.is_exist_rollup_id(&rollup_id) {
            return Ok(AddRollupResponse { success: false });
        }

        rollup_id_list_model.add_rollup_id(rollup_id.clone());
        rollup_id_list_model.update()?;

        let rollup = Rollup::new(
            parameter.rollup_id,
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

        let rollup_model =
            RollupModel::new(rollup, sequencing_info_key, parameter.cluster_id.clone());
        rollup_model.put()?;

        // TODO: get rollup_block_height from the rollup
        let rollup_metadata = RollupMetadata::new(0, TransactionOrder::from(0), OrderHash::new());

        context.set_rollup_state(
            rollup_id.clone(),
            RollupState::new(rollup_metadata.block_height()),
        );
        context.set_cluster_id(rollup_id.clone(), parameter.cluster_id);

        let rollup_metadata_model = RollupMetadataModel::new(rollup_id, rollup_metadata);
        rollup_metadata_model.put()?;

        Ok(AddRollupResponse { success: true })
    }
}
