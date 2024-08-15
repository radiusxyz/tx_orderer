use crate::{models::RollupMetadataModel, rpc::prelude::*, state::RollupState};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub rollup_id: RollupId,
    pub cluster_block_height: BlockHeight, // TODO
    pub rollup_block_height: BlockHeight,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let finalizing_block_height = context.get_block_height(&parameter.rollup_id)?;

        // TODO: verify rollup signature
        if finalizing_block_height != parameter.rollup_block_height {
            return Err(Error::InvalidBlockHeight.into());
        }

        let rollup_metadata_model = RollupMetadataModel::get(&parameter.rollup_id)?;
        let transaction_order = rollup_metadata_model.rollup_metadata().transaction_order();

        let cluster_id = context.get_cluster_id(&parameter.rollup_id)?;
        let cluster = context.get_cluster(&cluster_id)?;

        syncer::sync_block(
            cluster.clone(),
            parameter.rollup_id.clone(),
            parameter.cluster_block_height,
            parameter.rollup_block_height,
            transaction_order,
        );

        builder::finalize_block(
            parameter.rollup_id.clone(),
            cluster,
            finalizing_block_height,
            transaction_order,
        );

        let new_rollup_metadata = RollupMetadata::new(
            parameter.rollup_block_height + 1,
            TransactionOrder::new(0),
            OrderHash::new(),
        );

        context.set_rollup_state(
            parameter.rollup_id.clone(),
            RollupState::new(new_rollup_metadata.block_height()),
        );

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        rollup_metadata.update_rollup_metadata(new_rollup_metadata);
        rollup_metadata.update()?;

        Ok(SequencerStatus::Running)
    }
}
