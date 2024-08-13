use crate::{models::RollupMetadataModel, rpc::prelude::*};

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
        println!(
            "jaemin: finalizing_block_height: {:?}, parameter.rollup_block_height: {:?}",
            finalizing_block_height, parameter.rollup_block_height
        );
        if finalizing_block_height != parameter.rollup_block_height {
            return Err(Error::InvalidBlockHeight.into());
        }

        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        let transaction_order = RollupMetadataModel::get(&parameter.rollup_id)?
            .rollup_metadata()
            .transaction_order();
        // context.get_transaction_order(&parameter.rollup_id).await?;

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

        let new_rollup_metadata = RollupMetadata::new(
            parameter.rollup_block_height + 1,
            TransactionOrder::new(0),
            OrderHash::new(),
        );

        context
            .update_rollup_metadata(parameter.rollup_id.clone(), new_rollup_metadata.clone())
            .await;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        rollup_metadata.update_rollup_metadata(new_rollup_metadata);
        rollup_metadata.update()?;

        Ok(SequencerStatus::Running)
    }
}
