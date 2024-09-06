use super::SyncBlock;
use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub rollup_id: String,
    pub liveness_block_height: u64,
    pub rollup_block_height: u64,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata) => {
                let current_rollup_metadata =
                    rollup_metadata.issue_rollup_metadata(parameter.rollup_block_height);
                rollup_metadata.update()?;

                // Sync.
                Self::sync_block(&parameter, current_rollup_metadata.block_height());
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();
                    rollup_metadata.set_block_height(parameter.rollup_block_height);
                    RollupMetadataModel::put(&parameter.rollup_id, &rollup_metadata)?;

                    // Sync.
                    Self::sync_block(&parameter, rollup_metadata.block_height());
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    pub fn sync_block(parameter: &Self, transaction_order: u64) {
        let parameter = parameter.clone();

        tokio::spawn(async move {
            let rpc_parameter = SyncBlock {
                rollup_id: parameter.rollup_id.clone(),
                liveness_block_height: parameter.liveness_block_height,
                rollup_block_height: parameter.rollup_block_height,
                transaction_order: transaction_order,
            };

            let cluster_info =
                ClusterInfoModel::get(parameter.liveness_block_height, &parameter.rollup_id)
                    .unwrap();

            // Todo: Fire and forget.
        });
    }
}
