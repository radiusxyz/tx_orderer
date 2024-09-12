use crate::{
    rpc::{cluster::SyncRawTransaction, prelude::*},
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransactionMessage {
    rollup_id: String,
    raw_transaction: RawTransaction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    message: SendRawTransactionMessage,
    signature: Signature,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // // get rollup info for address and chain type
        // // verify siganture
        // parameter.signature.verify_signature(
        //     serialize_to_bincode(&parameter.message)?.as_slice(),
        //     parameter.message.address.as_slice(),
        //     parameter.message.chain_type,
        // )?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.message.rollup_id)?;

        let cluster_metadata = ClusterMetadataModel::get(
            &parameter.message.rollup_id,
            rollup_metadata.block_height(),
        )?;
        match cluster_metadata.is_leader() {
            true => {
                let transaction_order = rollup_metadata.issue_transaction_order();
                let order_hash = rollup_metadata
                    .issue_order_hash(&parameter.message.raw_transaction.raw_transaction_hash());
                let rollup_block_height = rollup_metadata.block_height();
                rollup_metadata.update()?;

                let order_commitment_data = OrderCommitmentData {
                    rollup_id: parameter.message.rollup_id.clone(),
                    block_height: rollup_block_height,
                    transaction_order,
                    previous_order_hash: order_hash,
                };
                let order_commitment = OrderCommitment {
                    data: order_commitment_data,
                    signature: vec![].into(), // Todo: Signature
                };

                RawTransactionModel::put(
                    &parameter.message.rollup_id,
                    rollup_block_height,
                    transaction_order,
                    parameter.message.raw_transaction.clone(),
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
                rollup_id: parameter.message.rollup_id,
                raw_transaction: parameter.message.raw_transaction,
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
