use crate::{
    rpc::{cluster::FinalizeBlockMessage, prelude::*},
    task::block_builder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub message: FinalizeBlockMessage,
    pub signature: Signature,
    pub transaction_count: u64,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = "sync_block";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "sync block - executor address: {:?}, rollup_id: {:?}, platform block height: {:?}, rollup block height: {:?}, transaction count: {:?}",
            parameter.message.executor_address.as_hex_string(),
            parameter.message.rollup_id,
            parameter.message.platform_block_height,
            parameter.message.rollup_block_height,
            parameter.transaction_count,
        );

        let rollup = Rollup::get(&parameter.message.rollup_id)?;

        // Verify the message.
        // parameter.signature.verify_message(
        //     rollup.platform().into(),
        //     &parameter.message,
        //     parameter.message.executor_address.clone(),
        // )?;

        let cluster = Cluster::get(
            rollup.platform(),
            rollup.service_provider(),
            rollup.cluster_id(),
            parameter.message.platform_block_height,
        )?;

        let next_rollup_block_height = parameter.message.rollup_block_height + 1;
        let is_leader = cluster.is_leader(next_rollup_block_height);

        match RollupMetadata::get_mut(&parameter.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                rollup_metadata.set_rollup_block_height(next_rollup_block_height);
                rollup_metadata.set_order_hash(OrderHash::default());
                rollup_metadata.set_transaction_order(0);
                rollup_metadata.set_is_leader(is_leader);
                rollup_metadata.set_platform_block_height(parameter.message.platform_block_height);

                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();

                    rollup_metadata.set_cluster_id(rollup.cluster_id());

                    rollup_metadata.set_rollup_block_height(next_rollup_block_height);
                    rollup_metadata.set_order_hash(OrderHash::default());
                    rollup_metadata.set_transaction_order(0);
                    rollup_metadata.set_is_leader(is_leader);
                    rollup_metadata
                        .set_platform_block_height(parameter.message.platform_block_height);

                    RollupMetadata::put(&rollup_metadata, &parameter.message.rollup_id)?;
                } else {
                    return Err(error.into());
                }
            }
        };

        block_builder(
            context.clone(),
            parameter.message.rollup_id.clone(),
            rollup.encrypted_transaction_type(),
            parameter.message.rollup_block_height,
            parameter.transaction_count,
            cluster,
        );

        Ok(())
    }
}
