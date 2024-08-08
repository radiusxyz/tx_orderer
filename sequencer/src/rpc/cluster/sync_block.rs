use crate::{models::RollupMetadataModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub rollup_id: RollupId,
    pub cluster_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster = context.get_cluster(&parameter.rollup_id).await?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata_model) => {
                let previous_rollup_block_number =
                    rollup_metadata_model.rollup_metadata().block_height();
                let previous_rollup_transaction_order =
                    rollup_metadata_model.rollup_metadata().transaction_order();

                let rollup_metadata =
                    RollupMetadata::new(parameter.rollup_block_height, 0.into(), OrderHash::new());
                rollup_metadata_model.update_rollup_metadata(rollup_metadata.clone());
                rollup_metadata_model.update()?;

                // update context rollup metadata
                context
                    .update_rollup_metadata(parameter.rollup_id.clone(), rollup_metadata)
                    .await;

                builder::finalize_block(
                    parameter.rollup_id,
                    cluster,
                    previous_rollup_block_number,
                    previous_rollup_transaction_order,
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

                    context
                        .update_rollup_metadata(parameter.rollup_id.clone(), rollup_metadata)
                        .await;
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}
