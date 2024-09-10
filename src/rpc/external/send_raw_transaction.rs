use crate::{
    rpc::{cluster::SyncRawTransaction, prelude::*},
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: RawTransaction,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        let cluster_metadata =
            ClusterMetadataModel::get(&parameter.rollup_id, rollup_metadata.block_height())?;
        match cluster_metadata.is_leader() {
            true => {
                let transaction_order = rollup_metadata.issue_transaction_order();
                let order_hash = rollup_metadata
                    .issue_order_hash(&parameter.raw_transaction.raw_transaction_hash());
                let rollup_block_height = rollup_metadata.block_height();
                rollup_metadata.update()?;

                let order_commitment_data = OrderCommitmentData {
                    rollup_id: parameter.rollup_id.clone(),
                    block_height: rollup_block_height,
                    transaction_order,
                    previous_order_hash: order_hash,
                };
                let order_commitment = OrderCommitment {
                    data: order_commitment_data,
                    signature: vec![].into(), // Todo: Signature
                };

                EncryptedTransactionModel::unencrypted_transaction();
                let encrypted_transaction_model =
                    EncryptedTransactionModel::unencrypted_transaction();
                encrypted_transaction_model.put(
                    &parameter.rollup_id,
                    rollup_block_height,
                    transaction_order,
                )?;

                let raw_transaction_model =
                    RawTransactionModel::new(parameter.raw_transaction.clone());
                raw_transaction_model.put(
                    &parameter.rollup_id,
                    rollup_block_height,
                    transaction_order,
                )?;

                // Sync Transaction
                Self::sync_raw_transaction(parameter, order_commitment.clone(), cluster_metadata);

                Ok(order_commitment)
            }
            false => {
                let leader_rpc_url = cluster_metadata.leader().ok_or(Error::EmptyLeaderRpcUrl)?;
                let client = RpcClient::new(leader_rpc_url)?;
                let response: OrderCommitment = client
                    .request(SendRawTransaction::METHOD_NAME, parameter.clone())
                    .await?;

                Ok(response)
            }
        }
    }

    pub fn sync_raw_transaction(
        parameter: Self,
        order_commitment: OrderCommitment,
        cluster_metadata: ClusterMetadata,
    ) {
        tokio::spawn(async move {
            let rpc_parameter = SyncRawTransaction {
                rollup_id: parameter.rollup_id,
                raw_transaction: parameter.raw_transaction,
                order_commitment,
            };

            for follower in cluster_metadata.followers() {
                let rpc_parameter = rpc_parameter.clone();

                tokio::spawn(async move {
                    let client = RpcClient::new(follower.unwrap()).unwrap();
                    let _ = client
                        .request::<SyncRawTransaction, ()>(
                            SyncRawTransaction::METHOD_NAME,
                            rpc_parameter,
                        )
                        .await
                        .unwrap();
                });
            }
        });
    }
}
