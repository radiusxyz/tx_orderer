use radius_sequencer_sdk::signature::{ChainType, PrivateKeySigner};

use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    rollup_id: String,
    raw_transaction: RawTransaction,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster_metadata = ClusterMetadataModel::get(&parameter.rollup_id)?;
        match cluster_metadata.is_leader() {
            true => {
                let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
                let transaction_order = rollup_metadata.issue_transaction_order();
                let order_hash = rollup_metadata
                    .issue_order_hash(&parameter.raw_transaction.raw_transaction_hash());
                let rollup_block_height = rollup_metadata.block_height();
                rollup_metadata.update()?;

                let order_commitment_data = OrderCommitmentData {
                    rollup_id: parameter.rollup_id,
                    block_height: rollup_block_height,
                    transaction_order,
                    previous_order_hash: order_hash,
                };

                let signing_key = context.config().signing_key();

                let sequencer_signer = ChainType::Ethereum
                    .create_signer_from_str(signing_key)
                    .unwrap();
                let order_commitment_signature = sequencer_signer
                    .sign_message(order_commitment_data.as_bytes().as_slice())
                    .unwrap();

                let order_commitment = OrderCommitment {
                    data: order_commitment_data,
                    signature: order_commitment_signature, // Use radius_sdk::signature::Signature;
                };

                EncryptedTransactionModel::unencrypted_transaction();
                let encrypted_transaction_model =
                    EncryptedTransactionModel::unencrypted_transaction();
                encrypted_transaction_model.put(
                    &parameter.rollup_id,
                    rollup_block_height,
                    transaction_order,
                )?;

                let raw_transaction_model = RawTransactionModel::new(parameter.raw_transaction);
                raw_transaction_model.put(
                    &parameter.rollup_id,
                    rollup_block_height,
                    transaction_order,
                )?;

                // Sync Transaction

                Ok(order_commitment)
            }
            false => {
                let leader_rpc_url = cluster_metadata.leader().ok_or(Error::EmptyLeaderRpcUrl)?;
                let client = RpcClient::new(leader_rpc_url)?;
                let response: OrderCommitment = client
                    .request(SendRawTransaction::METHOD_NAME, parameter)
                    .await?;

                Ok(response)
            }
        }
    }
}
