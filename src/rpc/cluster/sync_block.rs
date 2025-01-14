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

impl RpcParameter<AppState> for SyncBlock {
    type Response = ();

    fn method() -> &'static str {
        "sync_block"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "sync block - executor address: {:?}, rollup_id: {:?}, platform block height: {:?}, rollup block height: {:?}, transaction count: {:?}",
            self.message.executor_address.as_hex_string(),
            self.message.rollup_id,
            self.message.platform_block_height,
            self.message.rollup_block_height,
            self.transaction_count,
        );

        let rollup = Rollup::get(&self.message.rollup_id)?;

        // Verify the message.
        // parameter.signature.verify_message(
        //     rollup.platform.into(),
        //     &parameter.message,
        //     parameter.message.executor_address.clone(),
        // )?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.service_provider,
            &rollup.cluster_id,
            self.message.platform_block_height,
        )?;

        let next_rollup_block_height = self.message.rollup_block_height + 1;
        let signer = context.get_signer(rollup.platform).await.unwrap();
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == self.message.next_block_creator_address;

        // let is_leader = cluster.is_leader(next_rollup_block_height);

        match RollupMetadata::get_mut(&self.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                rollup_metadata.rollup_block_height = next_rollup_block_height;
                rollup_metadata.new_merkle_tree();
                rollup_metadata.is_leader = is_leader;
                rollup_metadata.leader_sequencer_rpc_info = cluster
                    .get_sequencer_rpc_info(&self.message.next_block_creator_address)
                    .unwrap();
                rollup_metadata.platform_block_height = self.message.platform_block_height;

                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();

                    rollup_metadata.cluster_id = rollup.cluster_id;
                    rollup_metadata.rollup_block_height = next_rollup_block_height;
                    rollup_metadata.is_leader = is_leader;
                    rollup_metadata.leader_sequencer_rpc_info = cluster
                        .get_sequencer_rpc_info(&self.message.next_block_creator_address)
                        .unwrap();
                    rollup_metadata.platform_block_height = self.message.platform_block_height;
                    rollup_metadata.new_merkle_tree();

                    RollupMetadata::put(&rollup_metadata, &self.message.rollup_id)?;
                } else {
                    return Err(error.into());
                }
            }
        };

        block_builder(
            context.clone(),
            self.message.rollup_id.clone(),
            self.message.block_creator_address.clone(),
            rollup.encrypted_transaction_type,
            self.message.rollup_block_height,
            self.transaction_count,
            cluster,
        );

        Ok(())
    }
}
