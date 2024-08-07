use crate::rpc::prelude::*;

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

        // TODO: verify rollup signature
        let finalizing_block_height = context.block_height(&parameter.rollup_id).await?;
        if finalizing_block_height != parameter.rollup_block_height {
            return Ok(SequencerStatus::Uninitialized); // TODO
        }

        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        let transaction_order = context.get_transaction_order(&parameter.rollup_id).await?;

        syncer::sync_block(
            cluster.clone(),
            parameter.rollup_id.clone(),
            parameter.cluster_block_height,
            parameter.rollup_block_height,
            transaction_order.clone(),
        );

        builder::finalize_block(
            parameter.rollup_id,
            cluster,
            finalizing_block_height,
            transaction_order,
        );

        Ok(SequencerStatus::Running)
    }
}
