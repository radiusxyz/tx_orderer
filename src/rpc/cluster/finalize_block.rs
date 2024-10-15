use tracing::info;

use crate::{
    rpc::{cluster::SyncBlock, prelude::*},
    task::block_builder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub executor_address: Address,
    pub rollup_id: String,
    pub platform_block_height: u64,
    pub rollup_block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub message: FinalizeBlockMessage,
    pub signature: Signature,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup = Rollup::get(&parameter.message.rollup_id)?;

        info!("finalize block - {:?}", parameter);

        // Verify the message.
        // parameter.signature.verify_message(
        //     rollup.platform().into(),
        //     &parameter.message,
        //     parameter.message.executor_address.clone(),
        // )?;

        // TODO:  check executor_address is in rollup executor addresses.

        let cluster = Cluster::get(
            rollup.platform(),
            rollup.service_provider(),
            rollup.cluster_id(),
            parameter.message.platform_block_height,
        )?;

        let next_rollup_block_height = parameter.message.rollup_block_height + 1;
        let is_leader = cluster.is_leader(next_rollup_block_height);

        let mut transaction_count = 0;
        match RollupMetadata::get_mut(&parameter.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                // TODO: check the block generated or generating.

                // if rollup_metadata.rollup_block_height() !=
                // parameter.message.rollup_block_height {     return
                // Err(Error::InvalidBlockHeight.into()); }

                transaction_count = rollup_metadata.transaction_order();

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

        // Sync.
        Self::sync_block(&parameter, transaction_count, cluster.clone());

        block_builder(
            context.clone(),
            parameter.message.rollup_id.clone(),
            rollup.encrypted_transaction_type(),
            parameter.message.rollup_block_height,
            transaction_count,
            cluster,
        );

        Ok(())
    }

    pub fn sync_block(parameter: &Self, transaction_count: u64, cluster: Cluster) {
        let parameter = parameter.clone();

        tokio::spawn(async move {
            let rpc_parameter = SyncBlock {
                message: parameter.message,
                signature: parameter.signature,
                transaction_count,
            };

            for sequencer_rpc_url in cluster.get_others_rpc_url_list() {
                let rpc_parameter = rpc_parameter.clone();

                if let Some(sequencer_rpc_url) = sequencer_rpc_url {
                    tokio::spawn(async move {
                        let client = RpcClient::new(sequencer_rpc_url).unwrap();
                        let _ = client
                            .request::<SyncBlock, ()>(SyncBlock::METHOD_NAME, rpc_parameter.clone())
                            .await;
                    });
                }
            }
        });
    }
}
