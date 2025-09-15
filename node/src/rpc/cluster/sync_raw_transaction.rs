use std::time::{SystemTime, UNIX_EPOCH};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub rollup_id: RollupId,

    pub batch_number: u64,
    pub transaction_order: u64,

    pub raw_transaction: RawTransaction,
    pub order_commitment: OrderCommitment,

    pub is_direct_sent: bool,
}

impl RpcParameter<AppState> for SyncRawTransaction {
    type Response = ();

    fn method() -> &'static str {
        "sync_raw_transaction"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let start_sync_raw_transaction_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        let rollup_id = self.rollup_id.clone();
        let rollup = Rollup::get(&rollup_id).map_err(|error| {
            tracing::error!("Failed to get rollup: {:?}", error);
            Error::RollupNotFound
        })?;

        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )
        .map_err(|error| {
            tracing::error!("Failed to get cluster metadata: {:?}", error);
            Error::ClusterMetadataNotFound
        })?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_metadata.platform_block_height,
        )?;

        // Verify the leader signature
        let is_valid_order_commitment = match self.order_commitment {
            OrderCommitment::Single(ref single_order_commitment) => match single_order_commitment {
                SingleOrderCommitment::Sign(sign_order_commitment) => {
                    let signer_address =
                        sign_order_commitment.get_signer_address(rollup.platform.into());

                    let tx_orderer_address_list = cluster.get_tx_orderer_address_list();

                    let leader_tx_orderer_address = tx_orderer_address_list
                        .iter()
                        .find(|&tx_orderer_address| signer_address == *tx_orderer_address);

                    leader_tx_orderer_address.is_some()
                }
                SingleOrderCommitment::TransactionHash(_) => true,
            },
            OrderCommitment::Bundle(_bundle) => {
                todo!("Handle bundle order commitment");
            }
        };

        if !is_valid_order_commitment {
            return Err(Error::InvalidOrderCommitment.into());
        }

        let transaction_hash = self.raw_transaction.raw_transaction_hash();

        RawTransactionModel::put_with_transaction_hash(
            &rollup_id,
            &transaction_hash,
            self.raw_transaction.clone(),
            self.is_direct_sent,
        )
        .map_err(|error| {
            tracing::error!("Failed to put raw transaction with hash: {:?}", error);
            Error::Database(error)
        })?;

        RawTransactionModel::put(
            &rollup_id,
            self.batch_number,
            self.transaction_order,
            self.raw_transaction.clone(),
            self.is_direct_sent,
        )
        .map_err(|error| {
            tracing::error!("Failed to put raw transaction: {:?}", error);
            Error::Database(error)
        })?;

        self.order_commitment
            .put(&rollup_id, self.batch_number, self.transaction_order)
            .map_err(|error| {
                tracing::error!("Failed to put order commitment: {:?}", error);
                Error::Database(error)
            })?;

        CanProvideTransactionInfo::add_can_provide_transaction_orders(
            &rollup_id,
            self.batch_number,
            vec![self.transaction_order],
        )?;

        let end_sync_raw_transaction_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        tracing::info!(
            "sync_raw_transaction - total take time: {:?}",
            end_sync_raw_transaction_time - start_sync_raw_transaction_time
        );

        Ok(())
    }
}
