use crate::{models::RollupMetadataModel, rpc::prelude::*, state::RollupState};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub rollup_id: RollupId,
    pub cluster_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = "sync_block";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster = context.get_cluster(&parameter.rollup_id)?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata_model) => {
                rollup_metadata_model.issue_new_block();
                rollup_metadata_model.update()?;

                // update context rollup state
                context.issue_new_block(&parameter.rollup_id)?;

                builder::finalize_block(
                    parameter.rollup_id,
                    cluster,
                    parameter.rollup_block_height,
                    parameter.transaction_order,
                );
            }
            Err(error) => {
                if error.is_none_type() {
                    let rollup_metadata = RollupMetadata::new(
                        parameter.rollup_block_height,
                        0.into(),
                        OrderHash::new(),
                    );
                    let rollup_metadata_model = RollupMetadataModel::new(
                        parameter.rollup_id.clone(),
                        rollup_metadata.clone(),
                    );
                    rollup_metadata_model.put()?;

                    context.set_rollup_state(
                        parameter.rollup_id,
                        RollupState::new(
                            cluster.cluster_id().clone(),
                            parameter.rollup_block_height,
                        ),
                    );
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}
