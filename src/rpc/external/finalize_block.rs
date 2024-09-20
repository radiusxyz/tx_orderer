use radius_sequencer_sdk::signature::Address;

use crate::{
    rpc::{
        cluster::{SyncBlock, SyncBlockMessage},
        prelude::*,
    },
    task::block_builder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    platform: Platform,
    // service_provider: ServiceProvider,
    // cluster_id: String,
    // chain_type: ChainType,
    address: Address,
    rollup_id: String,
    platform_block_height: u64,
    rollup_block_height: u64,
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
        let rollup = RollupModel::get(&parameter.message.rollup_id)?;

        // // verify siganture
        // parameter.signature.verify_message(
        //     rollup.rollup_type().into(),
        //     &parameter.message,
        //     parameter.message.address.clone(),
        // )?;

        let cluster =
            ClusterModel::get(rollup.cluster_id(), parameter.message.platform_block_height)?;

        let next_rollup_block_height = parameter.message.rollup_block_height + 1;
        let is_leader = cluster.is_leader(next_rollup_block_height);

        let mut transaction_counts = 0;
        match RollupMetadataModel::get_mut(&parameter.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                transaction_counts = rollup_metadata.transaction_order();

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

                    RollupMetadataModel::put(&parameter.message.rollup_id, &rollup_metadata)?;
                } else {
                    return Err(error.into());
                }
            }
        };

        // Sync.
        Self::sync_block(&parameter, transaction_counts, cluster);

        block_builder(
            parameter.message.rollup_id.clone(),
            parameter.message.rollup_block_height,
            transaction_counts,
            rollup.encrypted_transaction_type(),
            context.key_management_system_client().clone(),
            context.zkp_params(),
        );

        Ok(())
    }

    pub fn sync_block(parameter: &Self, transaction_order: u64, cluster: Cluster) {
        let parameter = parameter.clone();

        let sync_block_message = SyncBlockMessage {
            platform: parameter.message.platform.clone(),
            address: parameter.message.address.clone(),
            rollup_id: parameter.message.rollup_id.clone(),
            liveness_block_height: parameter.message.platform_block_height,
            rollup_block_height: parameter.message.rollup_block_height,
            transaction_order,
        };

        tokio::spawn(async move {
            let rpc_parameter = SyncBlock {
                message: sync_block_message.clone(),
                signature: parameter.signature.clone(),
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
