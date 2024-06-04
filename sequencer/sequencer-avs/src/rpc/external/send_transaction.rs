use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    transaction: Transaction,
}

#[async_trait]
impl RpcMethod for SendTransaction {
    type Response = ();

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
        Ok(())
    }
}

impl SendTransaction {
    async fn sync_transaction() {}

    async fn leader(self, cluster_metadata: ClusterMetadata) -> Result<OrderCommitment, RpcError> {
        // Issue order commitment
        let order_commitment =
            BlockMetadata::issue_order_commitment(cluster_metadata.rollup_block_number())?;

        // TODO: Spawn a syncer task to forward the transaction to other sequencers.
        // tokio::spawn(async move {});

        Ok(order_commitment)
    }

    async fn follower(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<OrderCommitment, RpcError> {
        match cluster_metadata.leader() {
            Some((leader_public_key, leader_address)) => {
                let rpc_client = RpcClient::new(leader_address, 5)?;
                let order_commitment: OrderCommitment = rpc_client.request(self).await?;
                Ok(order_commitment)
            }
            None => Err(Error::EmptyLeaderAddress.into()),
        }
    }
}
