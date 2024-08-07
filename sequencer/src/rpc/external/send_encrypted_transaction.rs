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
        let parameter = parameter.parse::<SendEncryptedTransaction>()?;

        // TODO: 1. verify encrypted_transaction

        // 2. Issue order_commitment
        let block_height = context.block_height(&parameter.rollup_id).await?;
        let transaction_order = context
            .get_current_transaction_order_and_increase_transaction_order(&parameter.rollup_id)
            .await?;
        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id.clone(),
            block_height,
            transaction_order: transaction_order.clone(),
            previous_order_hash: OrderHash::default(), // TODO
        };
        let order_commitment_signature = Signature::default(); // TODO
        let order_commitment = OrderCommitment {
            data: order_commitment_data,
            signature: order_commitment_signature,
        };

        // 3. Save encrypted_transaction
        let encrypted_transaction_model = EncryptedTransactionModel::new(
            parameter.encrypted_transaction.clone(),
            parameter.time_lock_puzzle.clone(),
        );
        encrypted_transaction_model.put(&parameter.rollup_id, &block_height, &transaction_order)?;

        // 4. Sync transaction
        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;
        syncer::sync_transaction(
            cluster,
            parameter.rollup_id,
            TransactionModel::Encrypted(encrypted_transaction_model),
            order_commitment.clone(),
        );

        Ok(order_commitment)
    }
}
