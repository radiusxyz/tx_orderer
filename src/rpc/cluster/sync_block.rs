use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub rollup_id: String,
    pub liveness_block_number: u64,
    pub rollup_block_number: u64,
    pub transaction_order: TransactionOrder,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = "sync_block";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata_model) => {
                let current_rollup_metadata = rollup_metadata_model.clone();
                rollup_metadata_model.issue_new_block();
                rollup_metadata_model.update()?;
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
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}
