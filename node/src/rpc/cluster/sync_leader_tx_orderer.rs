use std::time::{SystemTime, UNIX_EPOCH};

use radius_sdk::json_rpc::server::ProcessPriority;

use super::LeaderChangeMessage;
use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncLeaderTxOrderer {
    pub leader_change_message: LeaderChangeMessage,
    pub rollup_signature: Signature,

    pub batch_number: u64,
    pub transaction_order: u64,

    pub provided_batch_number: u64,
    pub provided_transaction_order: i64,
}

impl RpcParameter<AppState> for SyncLeaderTxOrderer {
    type Response = ();

    fn method() -> &'static str {
        "sync_leader_tx_orderer"
    }

    fn priority(&self) -> ProcessPriority {
        ProcessPriority::High
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let rollup_id = self.leader_change_message.rollup_id.clone();

        let start_sync_leader_tx_orderer_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        let rollup = Rollup::get(&rollup_id).map_err(|e| {
            tracing::error!("Failed to retrieve rollup: {:?}", e);
            Error::RollupNotFound
        })?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            self.leader_change_message.platform_block_height,
        )?;

        let signer = context.get_signer(rollup.platform).await.map_err(|_| {
            tracing::error!("Signer not found for platform {:?}", rollup.platform);
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

        let mut mut_cluster_metadata = ClusterMetadata::get_mut(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        mut_cluster_metadata.platform_block_height =
            self.leader_change_message.platform_block_height;
        mut_cluster_metadata.is_leader = is_leader;
        mut_cluster_metadata.leader_tx_orderer_rpc_info = Some(leader_tx_orderer_rpc_info.clone());
        mut_cluster_metadata.update()?;

        let mut mut_rollup_metadata = RollupMetadata::get_mut(&rollup_id)?;

        mut_rollup_metadata.batch_number = self.batch_number;
        mut_rollup_metadata.transaction_order = self.transaction_order;
        mut_rollup_metadata.provided_batch_number = self.provided_batch_number;
        mut_rollup_metadata.provided_transaction_order = self.provided_transaction_order;

        mut_rollup_metadata.update()?;

        let end_sync_leader_tx_orderer_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        tracing::info!(
            "sync_leader_tx_orderer - total take time: {:?} / self: {:?}",
            end_sync_leader_tx_orderer_time - start_sync_leader_tx_orderer_time,
            self
        );

        Ok(())
    }
}
