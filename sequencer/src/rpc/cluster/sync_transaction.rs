use crate::{
    models::{RollupMetadataModel, TransactionModel},
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

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        let mut rollup_metadatas = context.rollup_metadatas().await;

        let rollup_metadata = rollup_metadatas.get_mut(&parameter.rollup_id).unwrap();

        let new_order_hash = parameter.order_commitment.data.previous_order_hash;

        rollup_metadata.update_order_hash(new_order_hash.clone());
        rollup_metadata.increase_transaction_order();

        let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
        new_rollup_metadata_model.update_order_hash(new_order_hash);
        new_rollup_metadata_model.increase_transaction_order();

        rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model);
        rollup_metadata_model.update()?;

        // TODO: compare block height and transaction order with order commitment
        match parameter.transaction {
            TransactionModel::Encrypted(encrypted_transaction_model) => {
                encrypted_transaction_model.put(
                    &parameter.rollup_id,
                    &parameter.order_commitment.data.block_height,
                    &parameter.order_commitment.data.transaction_order,
                )?;
            }
            TransactionModel::Raw(raw_transaction_model) => {
                raw_transaction_model.put(
                    &parameter.rollup_id,
                    &parameter.order_commitment.data.block_height,
                    &parameter.order_commitment.data.transaction_order,
                )?;
            }
        }

        Ok(())
    }
}
