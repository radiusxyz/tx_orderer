use crate::{
    models::{
        EncryptedTransactionModel, RawTransactionModel, RollupMetadataModel, TransactionModel,
    },
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub rollup_id: RollupId,
    pub transaction: TransactionModel,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SyncRequest);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        let rollup_metadata = rollup_metadata_model.rollup_metadata();
        let block_height = rollup_metadata.block_height();
        // let mut transaction_order = rollup_metadata.transaction_order();
        let transaction_order = parameter.order_commitment.data.transaction_order.clone();

        let new_order_hash = parameter.order_commitment.data.previous_order_hash;

        if rollup_metadata.order_hash() != &new_order_hash {
            let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
            new_rollup_metadata_model.update_order_hash(new_order_hash);
            new_rollup_metadata_model.increase_transaction_order();

            rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model.clone());
            rollup_metadata_model.update()?;
            println!(
                "jaemin - update rollup metadata: transaction order: {:?},",
                new_rollup_metadata_model.transaction_order()
            );
        }
        // else {
        //     transaction_order = TransactionOrder::from(transaction_order.into_inner() - 1)
        // }

        match parameter.transaction {
            TransactionModel::Raw(raw_transaction_model) => {
                raw_transaction_model.put(
                    &parameter.rollup_id,
                    &block_height,
                    &transaction_order,
                )?;
            }
            TransactionModel::Encrypted(encrypted_transaction) => {
                encrypted_transaction.put(
                    &parameter.rollup_id,
                    &block_height,
                    &transaction_order,
                )?;
            }
        }

        Ok(())
    }
}
