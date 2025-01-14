use liveness::radius::LivenessClient;
use tracing::info;

use crate::{
    rpc::{cluster::SyncBlock, prelude::*},
    task::block_builder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub message: FinalizeBlockMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub executor_address: Address,
    pub block_creator_address: Address,
    pub next_block_creator_address: Address,
    pub rollup_id: String,
    pub platform_block_height: u64,
    pub rollup_block_height: u64,
}

impl RpcParameter<AppState> for FinalizeBlock {
    type Response = ();

    fn method() -> &'static str {
        "finalize_block"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        info!("finalize block - executor address: {:?} / block creator (sequencer) address: {:?} / rollup_id: {:?} / platform block height: {:?} / rollup block height: {:?}",
        self.message.executor_address.as_hex_string(),
        self.message.block_creator_address.as_hex_string(),
        self.message.rollup_id,
        self.message.platform_block_height,
        self.message.rollup_block_height,);

        // Verify the message.
        // self.signature.verify_message(

        //     rollup.platform.into(),
        //     &self.message,

        //     self.message.executor_address.clone(),

        // )?;

        // Check the executor address
        let rollup = Rollup::get(&self.message.rollup_id)?;

        // TODO: remove this comment /
        // In a rush to test, I couldn't add the executor address to the smart contract,
        // so I temporarily commented it out.
        // rollup
        //     .executor_address_list()
        //     .iter()
        //     .find(|&executor_address| self.message.executor_address ==

        // *executor_address)     .ok_or(Error::ExecutorAddressNotFound)?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.service_provider,
            &rollup.cluster_id,
            self.message.platform_block_height,
        );

        // TODO: update
        if cluster.is_err() {
            let liveness_client = context
                .get_liveness_client::<LivenessClient>(rollup.platform, rollup.service_provider)
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

            if platform_block_height - block_margin < self.message.platform_block_height {
                return Err(Error::InvalidPlatformBlockHeight)?;
            }

            // TODO:
            return Err(Error::ClusterNotFound)?;
        }

        let cluster = cluster.unwrap();

        let next_rollup_block_height = self.message.rollup_block_height + 1;

        let signer = context.get_signer(rollup.platform).await.unwrap();
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == self.message.next_block_creator_address;

        // let is_leader = cluster.is_leader(next_rollup_block_height);

        let mut transaction_count = 0;
        match RollupMetadata::get_mut(&self.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                transaction_count = rollup_metadata.transaction_order;

                rollup_metadata.rollup_block_height = next_rollup_block_height;
                rollup_metadata.platform_block_height = self.message.platform_block_height;

                rollup_metadata.is_leader = is_leader;
                rollup_metadata.leader_sequencer_rpc_info = cluster
                    .get_sequencer_rpc_info(&self.message.next_block_creator_address)
                    .unwrap();
                rollup_metadata.new_merkle_tree();

                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();

                    rollup_metadata.cluster_id = rollup.cluster_id;
                    rollup_metadata.rollup_block_height = next_rollup_block_height;
                    rollup_metadata.platform_block_height = self.message.platform_block_height;

                    rollup_metadata.is_leader = is_leader;
                    rollup_metadata.leader_sequencer_rpc_info = cluster
                        .get_sequencer_rpc_info(&self.message.next_block_creator_address)
                        .unwrap();
                    rollup_metadata.new_merkle_tree();

                    rollup_metadata.put(&self.message.rollup_id)?;
                } else {
                    return Err(error.into());
                }
            }
        };

        self.sync_block(transaction_count, cluster.clone());

        block_builder(
            context.clone(),
            self.message.rollup_id.clone(),
            self.message.block_creator_address.clone(),
            rollup.encrypted_transaction_type,
            self.message.rollup_block_height,
            transaction_count,
            cluster,
        );

        Ok(())
    }
}

impl FinalizeBlock {
    pub fn sync_block(&self, transaction_count: u64, cluster: Cluster) {
        let parameter = self.clone();

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
                    SyncBlock::method(),
                    &parameter,
                    Id::Null,
                )
                .await
                .unwrap();
        });
    }
}
