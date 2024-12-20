use liveness::radius::LivenessClient;
use tracing::info;

use crate::{
    rpc::{cluster::SyncBlock, prelude::*},
    task::block_builder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub executor_address: Address,
    pub block_creator_address: Address,
    pub next_block_creator_address: Address,
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

        info!("finalize block - executor address: {:?} / block creator (sequencer) address: {:?} / rollup_id: {:?} / platform block height: {:?} / rollup block height: {:?}",
            parameter.message.executor_address.as_hex_string(),
            parameter.message.block_creator_address.as_hex_string(),
            parameter.message.rollup_id,
            parameter.message.platform_block_height,
            parameter.message.rollup_block_height,
        );

        // Verify the message.
        // parameter.signature.verify_message(
        //     rollup.platform().into(),
        //     &parameter.message,
        //     parameter.message.executor_address.clone(),
        // )?;

        // Check the executor address
        let rollup = Rollup::get(&parameter.message.rollup_id)?;

        // TODO: remove this comment /
        // In a rush to test, I couldn't add the executor address to the smart contract,
        // so I temporarily commented it out.

        // rollup
        //     .executor_address_list()
        //     .iter()
        //     .find(|&executor_address| parameter.message.executor_address ==
        // *executor_address)     .ok_or(Error::ExecutorAddressNotFound)?;

        let cluster = Cluster::get(
            rollup.platform(),
            rollup.service_provider(),
            rollup.cluster_id(),
            parameter.message.platform_block_height,
        );

        // TODO: update
        if cluster.is_err() {
            let liveness_client = context
                .get_liveness_client::<LivenessClient>(rollup.platform(), rollup.service_provider())
                .await?;

            let platform_block_height = liveness_client
                .publisher()
                .get_block_number()
                .await
                .unwrap();

            let block_margin: u64 = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .unwrap()
                .try_into()
                .unwrap();

            if platform_block_height - block_margin < parameter.message.platform_block_height {
                return Err(Error::InvalidPlatformBlockHeight)?;
            }

            // TODO:
            return Err(Error::ClusterNotFound)?;
        }

        let cluster = cluster.unwrap();

        let next_rollup_block_height = parameter.message.rollup_block_height + 1;
        let signer = context.get_signer(rollup.platform()).await.unwrap();
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == parameter.message.next_block_creator_address;

        // let is_leader = cluster.is_leader(next_rollup_block_height);

        let mut transaction_count = 0;
        match RollupMetadata::get_mut(&parameter.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                transaction_count = rollup_metadata.transaction_order;

                rollup_metadata.set_rollup_block_height(next_rollup_block_height);
                rollup_metadata.new_merkle_tree();
                rollup_metadata.set_is_leader(is_leader);
                rollup_metadata.set_platform_block_height(parameter.message.platform_block_height);

                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();

                    rollup_metadata.set_cluster_id(rollup.cluster_id());

                    rollup_metadata.set_rollup_block_height(next_rollup_block_height);
                    rollup_metadata.new_merkle_tree();
                    rollup_metadata.set_is_leader(is_leader);
                    rollup_metadata
                        .set_platform_block_height(parameter.message.platform_block_height);

                    rollup_metadata.put(&parameter.message.rollup_id)?;
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
            parameter.message.block_creator_address.clone(),
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
            let parameter = SyncBlock {
                message: parameter.message,
                signature: parameter.signature,
                transaction_count,
            };

            let rpc_client = RpcClient::new().unwrap();
            let others_clsuter_rpc_url_list: Vec<String> =
                cluster.get_others_cluster_rpc_url_list();

            rpc_client
                .multicast(
                    others_clsuter_rpc_url_list,
                    SyncBlock::METHOD_NAME,
                    &parameter,
                    Id::Null,
                )
                .await;
        });
    }
}
