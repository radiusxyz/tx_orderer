use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub rollup_id: String,
    pub liveness_block_height: u64,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = "sync_block";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata) => {
                rollup_metadata.issue_rollup_metadata(parameter.rollup_block_height);
                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();
                    rollup_metadata.set_block_height(parameter.rollup_block_height);
                    RollupMetadataModel::put(&parameter.rollup_id, &rollup_metadata)?;
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}
