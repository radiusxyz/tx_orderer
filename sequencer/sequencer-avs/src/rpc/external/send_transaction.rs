use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    pub transaction: Transaction,
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
    async fn leader(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<<Self as RpcMethod>::Response, RpcError> {
        // Issue order commitment
        let order_commitment =
            BlockMetadata::issue_order_commitment(cluster_metadata.rollup_block_number())?;

        self.transaction.put(
            order_commitment.rollup_block_number(),
            order_commitment.transaction_order(),
        )?;

        // Spawn a transaction syncer task.
        transaction_syncer::init(self.transaction, order_commitment.clone(), cluster_metadata);
        Ok(order_commitment)
    }

    async fn follower(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<<Self as RpcMethod>::Response, RpcError> {
        let (_leader_public_key, leader_rpc_address) = cluster_metadata.leader();
        match leader_rpc_address {
            Some(rpc_address) => {
                let client = RpcClient::new(rpc_address, 5)?;
                client.request(self).await.map_err(|error| error.into())
            }
            None => Err(Error::EmptyLeaderAddress.into()),
        }
    }
}
