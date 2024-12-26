use crate::{
    rpc::{
        cluster::{SyncRawTransaction, SyncRawTransactionMessage},
        external::issue_order_commitment,
        prelude::*,
    },
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: RawTransaction,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "Send raw transaction: rollup_id: {:?}, raw_transaction: {:?}",
            parameter.rollup_id,
            parameter.raw_transaction
        );

        let rollup = Rollup::get(&parameter.rollup_id)?;

        // 2. Check is leader
        let mut rollup_metadata = RollupMetadata::get_mut(&parameter.rollup_id)?;
        let platform = rollup.platform;
        let service_provider = rollup.service_provider;
        let cluster_id = &rollup_metadata.cluster_id;
        let rollup_block_height = rollup_metadata.rollup_block_height;

        let cluster = Cluster::get(
            platform,
            service_provider,
            cluster_id,
            rollup_metadata.platform_block_height,
        )?;

        if rollup_metadata.is_leader {
            let (transaction_order, pre_merkle_path) = rollup_metadata
                .add_transaction_hash(parameter.raw_transaction.raw_transaction_hash().as_ref());
            rollup_metadata.update()?;

            let order_commitment = issue_order_commitment(
                context.clone(),
                rollup.platform,
                parameter.rollup_id.clone(),
                rollup.order_commitment_type,
                parameter.raw_transaction.raw_transaction_hash(),
                rollup_block_height,
                transaction_order,
                pre_merkle_path,
            )
            .await?;

            let transaction_hash = parameter.raw_transaction.raw_transaction_hash();

            RawTransactionModel::put_with_transaction_hash(
                &parameter.rollup_id,
                &transaction_hash,
                &parameter.raw_transaction,
            )?;

            RawTransactionModel::put(
                &parameter.rollup_id,
                rollup_block_height,
                transaction_order,
                &parameter.raw_transaction,
            )?;

            order_commitment.put(&parameter.rollup_id, rollup_block_height, transaction_order)?;

            // Sync Transaction
            sync_raw_transaction(
                cluster,
                context.clone(),
                rollup.platform,
                parameter.rollup_id.clone(),
                rollup_block_height,
                transaction_order,
                parameter.raw_transaction.clone(),
                order_commitment.clone(),
            );

            tracing::info!(
                "SendRawTransaction: order_commitment: {:?} / rollup_block_height: {:?} / transaction_order: {:?}",
                order_commitment,
                rollup_block_height,
                transaction_order
            );

            Ok(order_commitment)
        } else {
            let leader_external_rpc_url = rollup_metadata
                .leader_sequencer_rpc_info
                .external_rpc_url
                .clone()
                .unwrap();
            drop(rollup_metadata);

            let rpc_client = RpcClient::new()?;
            match rpc_client
                .request(
                    leader_external_rpc_url,
                    SendRawTransaction::METHOD_NAME,
                    &parameter,
                    Id::Null,
                )
                .await
            {
                Ok(response) => Ok(response),
                Err(error) => {
                    tracing::error!(
                        "Send raw transaction - leader external rpc error: {:?}",
                        error
                    );
                    Err(error.into())
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_raw_transaction(
    cluster: Cluster,
    context: Arc<AppState>,
    platform: Platform,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_order: u64,
    raw_transaction: RawTransaction,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let follower_rpc_url_list: Vec<String> =
            cluster.get_follower_cluster_rpc_url_list(rollup_block_height);

        if !follower_rpc_url_list.is_empty() {
            let message = SyncRawTransactionMessage {
                rollup_id,
                rollup_block_height,
                transaction_order,
                raw_transaction,
                order_commitment: Some(order_commitment),
            };
            let signature = context
                .get_signer(platform)
                .await
                .unwrap()
                .sign_message(&message)
                .unwrap();
            let rpc_parameter = SyncRawTransaction { message, signature };

            let rpc_client = RpcClient::new().unwrap();
            rpc_client
                .multicast(
                    follower_rpc_url_list,
                    SyncRawTransaction::METHOD_NAME,
                    &rpc_parameter,
                    Id::Null,
                )
                .await;
        }
    });
}
