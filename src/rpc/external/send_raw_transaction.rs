use tracing::info;

use crate::{
    rpc::{cluster::SyncRawTransaction, external::issue_order_commitment, prelude::*},
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
        let rollup = RollupModel::get(&parameter.rollup_id)?;

        // 2. Check is leader
        // TODO: error handling
        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        let platform = rollup.platform();
        let service_provider = rollup.service_provider();
        let cluster_id = rollup_metadata.cluster_id();
        let platform_block_height = rollup_metadata.platform_block_height();
        let rollup_block_height = rollup_metadata.rollup_block_height();

        let cluster = ClusterModel::get(
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        )
        .unwrap();

        if rollup_metadata.is_leader() {
            let transaction_order = rollup_metadata.transaction_order();
            rollup_metadata.increase_transaction_order();
            let previous_order_hash = rollup_metadata
                .update_order_hash(&parameter.raw_transaction.raw_transaction_hash());
            let current_order_hash = rollup_metadata.order_hash();
            rollup_metadata.update()?;

            let order_commitment = issue_order_commitment(
                context.clone(),
                rollup.platform(),
                parameter.rollup_id.clone(),
                rollup.order_commitment_type(),
                parameter.raw_transaction.raw_transaction_hash(),
                rollup_block_height,
                transaction_order,
                previous_order_hash,
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

            OrderCommitmentModel::put(
                &parameter.rollup_id,
                rollup_block_height,
                transaction_order,
                &order_commitment,
            )?;

            // Temporary block commitment
            BlockCommitmentModel::put(
                &parameter.rollup_id,
                rollup_block_height,
                transaction_order,
                &current_order_hash,
            )?;

            // Sync Transaction
            let follower_rpc_url_list = cluster.get_follower_rpc_url_list(rollup_block_height);

            sync_raw_transaction(
                parameter.rollup_id.clone(),
                parameter.raw_transaction.clone(),
                order_commitment.clone(),
                rollup_block_height,
                transaction_order,
                follower_rpc_url_list,
            );

            info!(
                "SendRawTransaction: order_commitment: {:?} / rollup_block_height: {:?} / transaction_order: {:?}",
                order_commitment,
                rollup_block_height,
                transaction_order
            );

            Ok(order_commitment)
        } else {
            let leader_rpc_url = cluster
                .get_leader_rpc_url(rollup_block_height)
                .ok_or(Error::EmptyLeaderRpcUrl)?;
            let client = RpcClient::new(leader_rpc_url)?;

            let response = client
                .request(SendRawTransaction::METHOD_NAME, parameter.clone())
                .await?;

            Ok(response)
        }
    }
}

pub fn sync_raw_transaction(
    rollup_id: String,
    raw_transaction: RawTransaction,

    order_commitment: OrderCommitment,

    rollup_block_height: u64,
    transaction_order: u64,

    follower_rpc_url_list: Vec<Option<String>>,
) {
    tokio::spawn(async move {
        let order_commitment = Some(order_commitment);

        let rpc_parameter = SyncRawTransaction {
            rollup_id,
            raw_transaction,

            transaction_order,
            rollup_block_height,

            order_commitment,
        };

        for follower_rpc_url in follower_rpc_url_list {
            let rpc_parameter = rpc_parameter.clone();

            tokio::spawn(async move {
                // TODO
                let client = RpcClient::new(follower_rpc_url.unwrap()).unwrap();
                let _ = client
                    .request::<SyncRawTransaction, ()>(
                        SyncRawTransaction::METHOD_NAME,
                        rpc_parameter,
                    )
                    .await
                    .unwrap();
            });
        }
    });
}
