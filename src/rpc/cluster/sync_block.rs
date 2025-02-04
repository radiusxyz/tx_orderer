use crate::{
    rpc::{cluster::FinalizeBlockMessage, prelude::*},
    task::follow_block,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub finalize_block_message: FinalizeBlockMessage,
    pub rollup_signature: Signature,

    pub transaction_count: u64,
    pub leader_sequencer_signature: Signature,
}

impl RpcParameter<AppState> for SyncBlock {
    type Response = ();

    fn method() -> &'static str {
        "sync_block"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "sync block - executor address: {:?}, rollup_id: {:?}, platform block height: {:?}, rollup block height: {:?}, transaction count: {:?}",
            self.finalize_block_message.executor_address.as_hex_string(),
            self.finalize_block_message.rollup_id,
            self.finalize_block_message.platform_block_height,
            self.finalize_block_message.rollup_block_height,
            self.transaction_count,
        );

        let rollup = Rollup::get(&self.finalize_block_message.rollup_id)?;
        let cluster = Cluster::get(
            rollup.platform,
            rollup.service_provider,
            &rollup.cluster_id,
            self.finalize_block_message.platform_block_height,
        )?;

        let next_rollup_block_height = self.finalize_block_message.rollup_block_height + 1;
        let signer = context.get_signer(rollup.platform).await.unwrap();
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == self.finalize_block_message.next_block_creator_address;

        match RollupMetadata::get_mut(&self.finalize_block_message.rollup_id) {
            Ok(mut rollup_metadata) => {
                rollup_metadata.rollup_block_height = next_rollup_block_height;
                rollup_metadata.is_leader = is_leader;
                rollup_metadata.leader_sequencer_rpc_info = cluster
                    .get_sequencer_rpc_info(&self.finalize_block_message.next_block_creator_address)
                    .unwrap();
                rollup_metadata.platform_block_height =
                    self.finalize_block_message.platform_block_height;

                context
                    .merkle_tree_manager()
                    .insert(&self.finalize_block_message.rollup_id, MerkleTree::new())
                    .await;
                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();
                    rollup_metadata.cluster_id = rollup.cluster_id;
                    rollup_metadata.rollup_block_height = next_rollup_block_height;
                    rollup_metadata.is_leader = is_leader;
                    rollup_metadata.leader_sequencer_rpc_info = cluster
                        .get_sequencer_rpc_info(
                            &self.finalize_block_message.next_block_creator_address,
                        )
                        .unwrap();
                    rollup_metadata.platform_block_height =
                        self.finalize_block_message.platform_block_height;

                    context
                        .merkle_tree_manager()
                        .insert(&self.finalize_block_message.rollup_id, MerkleTree::new())
                        .await;
                    rollup_metadata.put(&self.finalize_block_message.rollup_id)?;
                } else {
                    return Err(error.into());
                }
            }
        }

        follow_block(
            context.clone(),
            cluster,
            self.finalize_block_message,
            rollup.encrypted_transaction_type,
            self.transaction_count,
            self.leader_sequencer_signature,
        );

        Ok(())
    }
}
