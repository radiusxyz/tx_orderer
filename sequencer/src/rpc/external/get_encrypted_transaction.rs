use crate::{models::EncryptedTransactionModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransaction {
    pub rollup_id: RollupId,
    pub block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionResponse {
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: TimeLockPuzzle,
}

impl GetEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "get_encrypted_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetEncryptedTransactionResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let encrypted_transaction_model = EncryptedTransactionModel::get(
            &parameter.rollup_id,
            &parameter.block_height,
            &parameter.transaction_order,
        )?;

        Ok(GetEncryptedTransactionResponse {
            encrypted_transaction: encrypted_transaction_model.encrypted_transaction().clone(),
            time_lock_puzzle: encrypted_transaction_model.time_lock_puzzle().clone(),
        })
    }
}
