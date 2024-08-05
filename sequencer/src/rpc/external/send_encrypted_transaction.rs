use tracing::info;

use crate::{models::EncryptedTransactionModel, rpc::prelude::*, types::*};

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
        _context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        info!("SendEncryptedTransaction - {:?}", parameter);

        let parameter = parameter.parse::<SendEncryptedTransaction>()?;

        let encrypted_transaction_model = EncryptedTransactionModel::new(
            parameter.encrypted_transaction,
            parameter.time_lock_puzzle,
        );

        encrypted_transaction_model.put(&parameter.rollup_id, &0, &TransactionOrder::from(0))?;

        // TODO: verify encrypted_transaction

        // TODO
        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id,
            block_height: 0,
            transaction_order: TransactionOrder::from(0),
            previous_order_hash: OrderHash::default(),
        };

        // TODO
        let order_commitment = OrderCommitment {
            data: order_commitment_data,
            signature: Signature::default(),
        };

        Ok(order_commitment)
    }
}
