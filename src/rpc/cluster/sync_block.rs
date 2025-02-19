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

        let rollup = Rollup::get(&self.finalize_block_message.rollup_id).map_err(|e| {
            tracing::error!("Failed to retrieve rollup: {:?}", e);
            Error::RollupNotFound
        })?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.service_provider,
            &rollup.cluster_id,
            self.finalize_block_message.platform_block_height,
        );

        let cluster = if cluster.is_err() {
            tracing::warn!("Failed to retrieve cluster - cluster_id: {:?} / platform_block_height: {:?} / error: {:?}", 
            &rollup.cluster_id,
            self.finalize_block_message.platform_block_height,
            cluster.err());

            let liveness_service_manager_client: liveness_service_manager::radius::LivenessServiceManagerClient = context
                .get_liveness_service_manager_client::<liveness_service_manager::radius::LivenessServiceManagerClient>(
                    rollup.platform,
                    rollup.service_provider,
                )
                .await?;

            Cluster::sync_cluster(
                context.clone(),
                &rollup.cluster_id,
                &liveness_service_manager_client,
                self.finalize_block_message.platform_block_height,
            )
            .await?
        } else {
            cluster.unwrap()
        };

        let next_rollup_block_height = self.finalize_block_message.rollup_block_height + 1;
        let signer = context.get_signer(rollup.platform).await.map_err(|_| {
            tracing::error!("Signer not found for platform {:?}", rollup.platform);
            Error::SignerNotFound
        })?;
        let tx_orderer_address = signer.address().clone();
        let is_leader =
            tx_orderer_address == self.finalize_block_message.next_block_creator_address;

        let leader_tx_orderer_rpc_info = cluster
            .get_tx_orderer_rpc_info(&self.finalize_block_message.next_block_creator_address)
            .ok_or_else(|| {
                tracing::error!(
                    "TxOrderer RPC info not found for address {:?}",
                    self.finalize_block_message.next_block_creator_address
                );
                Error::TxOrdererInfoNotFound
            })?;

        match RollupMetadata::get_mut(&self.finalize_block_message.rollup_id) {
            Ok(mut rollup_metadata) => {
                rollup_metadata.rollup_block_height = next_rollup_block_height;
                rollup_metadata.transaction_order = 0;
                rollup_metadata.platform_block_height =
                    self.finalize_block_message.platform_block_height;
                rollup_metadata.is_leader = is_leader;
                rollup_metadata.max_gas_limit = rollup.max_gas_limit;
                rollup_metadata.current_gas = 0;
                rollup_metadata.leader_tx_orderer_rpc_info = leader_tx_orderer_rpc_info;

                context
                    .merkle_tree_manager()
                    .insert(&self.finalize_block_message.rollup_id, MerkleTree::new())
                    .await;
                rollup_metadata.update()?;
            }
            Err(error) => {
                if error.is_none_type() {
                    tracing::warn!("Rollup metadata not found");
                    let rollup_metadata = RollupMetadata {
                        rollup_block_height: next_rollup_block_height,
                        transaction_order: 0,
                        cluster_id: rollup.cluster_id,
                        platform_block_height: self.finalize_block_message.platform_block_height,
                        is_leader,
                        leader_tx_orderer_rpc_info,
                        max_gas_limit: rollup.max_gas_limit,
                        current_gas: 0,
                    };

                    context
                        .merkle_tree_manager()
                        .insert(&self.finalize_block_message.rollup_id, MerkleTree::new())
                        .await;
                    rollup_metadata.put(&self.finalize_block_message.rollup_id)?;
                } else {
                    tracing::error!("Failed to retrieve rollup metadata: {:?}", error);
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
