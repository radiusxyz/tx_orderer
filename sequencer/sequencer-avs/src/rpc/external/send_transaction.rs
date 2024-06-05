use crate::rpc::{external::SyncTransaction, prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    transaction: Transaction,
}

#[async_trait]
impl RpcMethod for SendTransaction {
    type Response = OrderCommitment;

    fn method_name() -> &'static str {
        stringify!(SendTransaction)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => match cluster_metadata.is_leader() {
                true => self.leader(cluster_metadata).await,
                false => self.follower(cluster_metadata).await,
            },
            Err(error) => {
                // Return `Error::Uninitialized` to the user if the sequencer has not
                // received any close block request from the rollup previously.
                match error.kind() {
                    database::ErrorKind::KeyDoesNotExist => Err(Error::Uninitialized.into()),
                    _others => Err(error.into()),
                }
            }
        }
    }
}

impl SendTransaction {
    fn syncer(processed_transaction: ProcessedTransaction, cluster_metadata: ClusterMetadata) {
        let rpc_method = SyncTransaction {
            transaction: processed_transaction,
        };

        // Fire and forget.
        tokio::spawn(async move {
            for (_public_key, rpc_address) in cluster_metadata.into_followers() {
                if let Some(rpc_address) = rpc_address {
                    let rpc_method = rpc_method.clone();
                    tokio::spawn(async move {
                        let rpc_client = RpcClient::new(rpc_address, 1).unwrap();
                        let _ = rpc_client.request(rpc_method).await;
                    });
                }
            }
        });
    }

    async fn leader(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<<Self as RpcMethod>::Response, RpcError> {
        // Issue order commitment
        let order_commitment =
            BlockMetadata::issue_order_commitment(cluster_metadata.rollup_block_number())?;

        let processed_transaction =
            ProcessedTransaction::new(order_commitment.clone(), self.transaction);
        processed_transaction.put()?;

        // Spawn a syncer task to forward the transaction to other sequencers.
        Self::syncer(processed_transaction, cluster_metadata);

        Ok(order_commitment)
    }

    async fn follower(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<<Self as RpcMethod>::Response, RpcError> {
        let (_leader_public_key, leader_rpc_address) = cluster_metadata.leader();
        if let Some(rpc_address) = leader_rpc_address {
            let client = RpcClient::new(rpc_address, 5)?;
            client.request(self).await.map_err(|error| error.into())
        } else {
            Err(Error::EmptyLeaderAddress.into())
        }
    }
}
