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
            return Err(Error::InvalidBlockHeight.into());
        }

        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        let transaction_order = context.get_transaction_order(&parameter.rollup_id).await?;

        println!("jaemin: transaction_order: {:?}", transaction_order);

        syncer::sync_block(
            cluster.clone(),
            parameter.rollup_id.clone(),
            parameter.cluster_block_height,
            parameter.rollup_block_height,
            transaction_order.clone(),
        );

        builder::finalize_block(
            parameter.rollup_id.clone(),
            cluster,
            finalizing_block_height,
            transaction_order,
        );

        context
            .update_rollup_metadata(
                parameter.rollup_id,
                RollupMetadata::new(
                    parameter.rollup_block_height + 1,
                    TransactionOrder::new(0),
                    OrderHash::new(),
                ),
            )
            .await;

        Ok(SequencerStatus::Running)
    }
}
