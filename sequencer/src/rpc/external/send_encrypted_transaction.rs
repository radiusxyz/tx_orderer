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

        encrypted_transaction_model.put(&parameter.rollup_id, &0, &TransactionOrder::from(0))?;

        // TODO: verify encrypted_transaction

        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        let transaction_model = TransactionModel::Encrypted(encrypted_transaction_model);

        // TODO
        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id.clone(),
            block_height: 0,
            transaction_order: TransactionOrder::from(0),
            previous_order_hash: OrderHash::default(),
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
