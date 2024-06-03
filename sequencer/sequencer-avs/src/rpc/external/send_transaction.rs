use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    transaction: Transaction,
}

#[async_trait]
impl RpcMethod for SendTransaction {
    type Response = ClusterStatus;

    fn method_name() -> &'static str {
        stringify!(SendTransaction)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match database().get::<&'static str, Cluster>(&CURRENT_CLUSTER) {
            Ok(cluster) => {
                // Load the block data based on the current cluster.
                let block_key =
                    BlockKey::new(cluster.ssal_block_number(), cluster.rollup_block_number());
                let mut block: Lock<Block> = database().get_mut(&block_key)?;

                // Get the transaction order.
                let transaction_order = block.transaction_order();
                block.commit()?;

                // Make a new order commitment.
                let order_commitment =
                    OrderCommitment::new(cluster.rollup_block_number(), transaction_order);

                // Spawn a transaction syncer.
                // transaction_syncer(cluster, self.transaction);

                Ok(ClusterStatus::Initialized(order_commitment))
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    // If the current cluster does not exist, return the uninitialized status to the user.
                    Ok(ClusterStatus::Uninitialized)
                } else {
                    // Return the error for other error cases.
                    Err(error.into())
                }
            }
        }
    }
}
