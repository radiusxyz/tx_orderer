use crate::{
    models::{RawTransactionModel, RollupMetadataModel, TransactionModel},
    rpc::prelude::*,
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    rollup_id: RollupId,
    raw_transaction: RawTransaction,
}

// TODO(jaemin): Check leader verification for order commitment
impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<SendRawTransaction>()?;

        let block_height = context.get_block_height(&parameter.rollup_id)?;

        let cluster_id = context.get_cluster_id(&parameter.rollup_id)?;
        let cluster = context.get_cluster(&cluster_id)?;
        let is_leader = cluster.is_leader(block_height).await;

        // forward to leader
        if !is_leader {
            let leader_rpc_client = cluster.get_leader_rpc_client(block_height).await;
            return leader_rpc_client
                .send_raw_transaction(parameter)
                .await
                .map_err(RpcError::from);
        }

        // 2. Issue order_commitment

        let raw_transaction_hash = parameter.raw_transaction.raw_transaction_hash();

        let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        let transaction_order = rollup_metadata_model.rollup_metadata().transaction_order();

        let previous_order_hash = rollup_metadata_model.rollup_metadata().order_hash();
        let issued_order_hash = previous_order_hash.issue_order_hash(&raw_transaction_hash);

        let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
        new_rollup_metadata_model.update_order_hash(issued_order_hash.clone());
        new_rollup_metadata_model.increase_transaction_order();

        rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model);
        rollup_metadata_model.update()?;

        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id.clone(),
            block_height,
            transaction_order,
            previous_order_hash: issued_order_hash,
        };

        let order_commitment_signature = Signature::default(); // TODO
        let order_commitment = OrderCommitment {
            data: order_commitment_data,
            signature: order_commitment_signature,
        };

        // 3. Save raw_transaction
        // Todo: change waiting decrypted raw transaction
        let raw_transaction_model = RawTransactionModel::new(parameter.raw_transaction);
        raw_transaction_model.put(&parameter.rollup_id, &block_height, &transaction_order)?;

        // 4. Sync transaction
        syncer::sync_transaction(
            cluster,
            parameter.rollup_id,
            TransactionModel::Raw(raw_transaction_model),
            order_commitment.clone(),
        );

        Ok(order_commitment)
    }
}
