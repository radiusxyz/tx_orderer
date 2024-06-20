use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    pub transaction: UserTransaction,
}

impl SendTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SendTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let database = context.database();

        match ClusterMetadata::get(&database) {
            Ok(cluster_metadata) => match cluster_metadata.is_leader {
                true => parameter.leader_handler(&database, cluster_metadata).await,
                false => parameter.follower_handler(cluster_metadata).await,
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

    async fn leader_handler(
        self,
        database: &Database,
        cluster_metadata: ClusterMetadata,
    ) -> Result<OrderCommitment, RpcError> {
        // Issue order commitment
        let order_commitment =
            BlockMetadata::issue_order_commitment(database, cluster_metadata.rollup_block_number)?;

        self.transaction.put(
            database,
            order_commitment.rollup_block_number(),
            order_commitment.transaction_order(),
        )?;

        // Spawn a transaction syncer task.
        syncer::sync_user_transaction(self.transaction, order_commitment.clone(), cluster_metadata);

        Ok(order_commitment)
    }

    async fn follower_handler(
        self,
        cluster_metadata: ClusterMetadata,
    ) -> Result<OrderCommitment, RpcError> {
        let (_leader_address, leader_rpc_url) = cluster_metadata.leader();

        if let Some(rpc_url) = leader_rpc_url {
            let client = RpcClient::new(rpc_url, 5)?;
            client
                .request(Self::METHOD_NAME, self)
                .await
                .map_err(|error| error.into())
        } else {
            Err(Error::EmptyLeaderAddress.into())
        }
    }
}
