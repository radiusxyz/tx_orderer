use radius_sequencer_sdk::signature::Address;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlockMessage {
    pub platform: Platform,
    // service_provider: ServiceProvider,
    // cluster_id: String,
    // chain_type: ChainType,
    pub address: Address,
    pub rollup_id: String,
    pub liveness_block_height: u64,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub message: SyncBlockMessage,
    pub signature: Signature,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = "sync_block";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // TODO:
        // match RollupMetadataModel::get_mut(&parameter.rollup_id) {
        //     Ok(mut rollup_metadata) => {
        //         rollup_metadata.new_rollup_metadata(parameter.rollup_block_height);
        //         rollup_metadata.update()?;

        //         rollup_metadata
        //     }
        //     Err(error) => {
        //         if error.is_none_type() {
        //             let mut rollup_metadata = RollupMetadata::default();

        //             rollup_metadata.set_block_height(parameter.rollup_block_height);

        //             RollupMetadataModel::put(&parameter.rollup_id,
        // &rollup_metadata)?;

        //             rollup_metadata
        //         } else {
        //             return Err(error.into());
        //         }
        //     }
        // }

        Ok(())
    }
}
