use tracing::info;

use crate::{rpc::prelude::*, task::build_block};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub finalize_block_message: FinalizeBlockMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub rollup_id: String,
    pub executor_address: Address,

    pub platform_block_height: u64,
    pub rollup_block_height: u64,

    pub block_creator_address: Address,
    pub next_block_creator_address: Address,
}

impl RpcParameter<AppState> for FinalizeBlock {
    type Response = ();

    fn method() -> &'static str {
        "finalize_block"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        info!("finalize block - executor address: {:?} / block creator (sequencer) address: {:?} / rollup_id: {:?} / platform block height: {:?} / rollup block height: {:?}",
        self.finalize_block_message.executor_address.as_hex_string(),
        self.finalize_block_message.block_creator_address.as_hex_string(),
        self.finalize_block_message.rollup_id,
        self.finalize_block_message.platform_block_height,
        self.finalize_block_message.rollup_block_height,);

        // Verify the message.
        // self.signature.verify_message(
        //     rollup.platform.into(),
        //     &self.message,
        //     self.message.executor_address.clone(),
        // )?;

        // Check the executor address
        let rollup = context
            .get_rollup(&self.finalize_block_message.rollup_id)
            .await?;

        // TODO: remove this comment /
        // In a rush to test, I couldn't add the executor address to the smart contract,
        // so I temporarily commented it out.
        // rollup
        //     .executor_address_list()
        //     .iter()
        //     .find(|&executor_address| self.message.executor_address ==

        // *executor_address)     .ok_or(Error::ExecutorAddressNotFound)?;

        let cluster = context
            .get_cluster(
                rollup.platform,
                rollup.service_provider,
                &rollup.cluster_id,
                self.finalize_block_message.platform_block_height,
            )
            .await
            .map_err(|_| Error::ClusterNotFound)?;

        let transaction_count = self
            .finalize_block(context.clone(), &cluster, &rollup)
            .await?;

        build_block(
            context,
            cluster,
            self.finalize_block_message,
            self.signature,
            rollup.encrypted_transaction_type.clone(),
            transaction_count,
        );

        Ok(())
    }
}

impl FinalizeBlock {
    pub async fn finalize_block(
        &self,
        context: AppState,
        cluster: &Cluster,
        rollup: &Rollup,
    ) -> Result<u64, RpcError> {
        let next_rollup_block_height = self.finalize_block_message.rollup_block_height + 1;

        let signer = context.get_signer(rollup.platform).await?;
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == self.finalize_block_message.next_block_creator_address;

        let mut transaction_count = 0;

        if let Ok(mut locked_rollup_metadata) = context
            .get_mut_rollup_metadata(&self.finalize_block_message.rollup_id)
            .await
        {
            transaction_count = locked_rollup_metadata.transaction_order;

            self.update_rollup_metadata(
                &mut locked_rollup_metadata,
                next_rollup_block_height,
                is_leader,
                cluster,
            )?;
        } else {
            let mut rollup_metadata = RollupMetadata::default();

            rollup_metadata.cluster_id = rollup.cluster_id.to_string();
            self.update_rollup_metadata(
                &mut rollup_metadata,
                next_rollup_block_height,
                is_leader,
                cluster,
            )?;

            rollup_metadata.put(&self.finalize_block_message.rollup_id)?;
            context
                .add_rollup_metadata(&self.finalize_block_message.rollup_id, rollup_metadata)
                .await?;
        }

        Ok(transaction_count)
    }

    fn update_rollup_metadata(
        &self,
        rollup_metadata: &mut RollupMetadata,
        next_rollup_block_height: u64,
        is_leader: bool,
        cluster: &Cluster,
    ) -> Result<(), RpcError> {
        rollup_metadata.rollup_block_height = next_rollup_block_height;
        rollup_metadata.platform_block_height = self.finalize_block_message.platform_block_height;
        rollup_metadata.is_leader = is_leader;

        if let Some(sequencer_rpc_info) =
            cluster.get_sequencer_rpc_info(&self.finalize_block_message.next_block_creator_address)
        {
            rollup_metadata.leader_sequencer_rpc_info = sequencer_rpc_info;
        } else {
            tracing::error!("Sequencer RPC info not found");
            return Err(Error::SignerNotFound)?;
        }

        rollup_metadata.new_merkle_tree();
        Ok(())
    }
}
