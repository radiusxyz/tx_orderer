use tracing::info;

use crate::{
    models::{EncryptedTransactionModel, TransactionModel},
    rpc::prelude::*,
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransaction {
    rollup_id: ClusterId,
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: TimeLockPuzzle,
}

impl SendEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "send_encrypted_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        info!("SendEncryptedTransaction - {:?}", parameter);

        let parameter = parameter.parse::<SendEncryptedTransaction>()?;

        let encrypted_transaction_model = EncryptedTransactionModel::new(
            parameter.encrypted_transaction.clone(),
            parameter.time_lock_puzzle.clone(),
        );

        // TODO: verify encrypted_transaction

        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        let block_height = context.block_height(&parameter.rollup_id).await?;
        let transaction_order = context
            .get_current_transaction_order_and_increase_transaction_order(&parameter.rollup_id)
            .await?;

        encrypted_transaction_model.put(&parameter.rollup_id, &block_height, &transaction_order)?;

        let transaction_model = TransactionModel::Encrypted(encrypted_transaction_model);

        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id.clone(),
            block_height,
            transaction_order,
            previous_order_hash: OrderHash::default(), // TODO
        };

        // TODO
        let order_commitment = OrderCommitment {
            data: order_commitment_data,
            signature: Signature::default(),
        };

        syncer::sync_transaction(
            cluster,
            parameter.rollup_id,
            transaction_model,
            order_commitment.clone(),
        );

        Ok(order_commitment)
    }
}
