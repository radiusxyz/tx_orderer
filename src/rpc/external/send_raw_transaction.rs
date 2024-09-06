use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    rollup_id: String,
    raw_transaction: RawTransaction,
}

// impl SendRawTransaction {
//     pub const METHOD_NAME: &'static str = "send_raw_transaction";

//     pub async fn handler(
//         parameter: RpcParameter,
//         context: Arc<AppState>,
//     ) -> Result<OrderCommitment, RpcError> {
//         let parameter = parameter.parse::<SendRawTransaction>()?;

//         let rollup_block_height = context.get_block_height(&parameter.rollup_id)?;

//         let cluster_id = context.get_cluster_id(&parameter.rollup_id)?;
//         let cluster = context.get_cluster(&cluster_id)?;
//         let is_leader = cluster.is_leader(rollup_block_height).await;

//         // forward to leader
//         if !is_leader {
//             let leader_rpc_client = cluster.get_leader_rpc_client(rollup_block_height).await;
//             return leader_rpc_client
//                 .send_raw_transaction(parameter)
//                 .await
//                 .map_err(RpcError::from);
//         }

//         // 2. Issue order_commitment

//         let raw_transaction_hash = parameter.raw_transaction.raw_transaction_hash();

//         let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

//         let transaction_order = rollup_metadata_model.rollup_metadata().transaction_order();

//         let previous_order_hash = rollup_metadata_model.rollup_metadata().order_hash();
//         let issued_order_hash = previous_order_hash.issue_order_hash(&raw_transaction_hash);

//         let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
//         new_rollup_metadata_model.update_order_hash(issued_order_hash.clone());
//         new_rollup_metadata_model.increase_transaction_order();

//         rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model);
//         rollup_metadata_model.update()?;

//         let order_commitment_data = OrderCommitmentData {
//             rollup_id: parameter.rollup_id.clone(),
//             block_height: rollup_block_height,
//             transaction_order,
//             previous_order_hash: issued_order_hash,
//         };

//         let order_commitment_signature = Signature::default(); // TODO
//         let order_commitment = OrderCommitment {
//             data: order_commitment_data,
//             signature: order_commitment_signature,
//         };

//         // 3. Save empty encrypted transaction
//         let encrypted_transaction_model = EncryptedTransactionModel::unencrypted_transaction();
//         encrypted_transaction_model.put(
//             &parameter.rollup_id,
//             &rollup_block_height,
//             &transaction_order,
//         )?;

//         // 4. Save raw_transaction
//         let raw_transaction_model = RawTransactionModel::new(parameter.raw_transaction);
//         raw_transaction_model.put(
//             &parameter.rollup_id,
//             &rollup_block_height,
//             &transaction_order,
//         )?;

//         // 5. Sync empty encrypted transaction
//         syncer::sync_transaction(
//             cluster.clone(),
//             parameter.rollup_id.clone(),
//             TransactionModel::Encrypted(encrypted_transaction_model),
//             order_commitment.clone(),
//         );

//         // 6. Sync raw transaction
//         syncer::sync_transaction(
//             cluster,
//             parameter.rollup_id,
//             TransactionModel::Raw(raw_transaction_model),
//             order_commitment.clone(),
//         );

//         Ok(order_commitment)
//     }
// }
