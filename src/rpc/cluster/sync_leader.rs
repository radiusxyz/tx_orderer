use super::LeaderChangeMessage;
use crate::rpc::prelude::*;

const LOG_TARGET: &str = "rpc::cluster::sync_block";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncLeaderTxOrderer {
    pub leader_change_message: LeaderChangeMessage,
    pub rollup_signature: Signature,
}

impl RpcParameter<AppState> for SyncLeaderTxOrderer {
    type Response = ();

    fn method() -> &'static str {
        "sync_leader_tx_orderer"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            target: LOG_TARGET,
            "sync leader tx orderer - executor address: {:?}, rollup_id: {:?}, platform block height: {:?}",            self.leader_change_message.executor_address.as_hex_string(),
            self.leader_change_message.rollup_id,
            self.leader_change_message.platform_block_height,
        );

        let rollup = Rollup::get(&self.leader_change_message.rollup_id).map_err(|e| {
            tracing::error!("Failed to retrieve rollup: {:?}", e);
            Error::RollupNotFound
        })?;

        let cluster = match Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            self.leader_change_message.platform_block_height,
        ) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    "Failed to retrieve cluster - cluster_id: {:?} / platform_block_height: {:?} / error: {:?}",
                    &rollup.cluster_id,
                    self.leader_change_message.platform_block_height,
                    e
                );

                let liveness_service_manager_client: liveness_service_manager::radius::LivenessServiceManagerClient = context
                    .get_liveness_service_manager_client::<liveness_service_manager::radius::LivenessServiceManagerClient>(
                        rollup.platform,
                        rollup.liveness_service_provider,
                    )
                    .await?;

                Cluster::sync_cluster(
                    context.clone(),
                    &rollup.cluster_id,
                    &liveness_service_manager_client,
                    self.leader_change_message.platform_block_height,
                )
                .await?
            }
        };

        let signer = context.get_signer(rollup.platform).await.map_err(|_| {
            tracing::error!(
                target: LOG_TARGET,
                "Signer not found for platform {:?}",
                rollup.platform
            );
            Error::SignerNotFound
        })?;
        let tx_orderer_address = signer.address().clone();
        let is_leader =
            tx_orderer_address == self.leader_change_message.next_leader_tx_orderer_address;

        let leader_tx_orderer_rpc_info = cluster
            .get_tx_orderer_rpc_info(&self.leader_change_message.next_leader_tx_orderer_address)
            .ok_or_else(|| {
                tracing::error!(
                    "TxOrderer RPC info not found for address {:?}",
                    self.leader_change_message.next_leader_tx_orderer_address
                );
                Error::TxOrdererInfoNotFound
            })?;

        let mut cluster_metadata = ClusterMetadata::get_mut(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        cluster_metadata.platform_block_height = self.leader_change_message.platform_block_height;
        cluster_metadata.is_leader = is_leader;
        cluster_metadata.leader_tx_orderer_rpc_info = Some(leader_tx_orderer_rpc_info.clone());
        cluster_metadata.update()?;

        Ok(())
    }
}
