use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use radius_sdk::{json_rpc::client::Priority, signature::Address};
use tokio::{sync::mpsc::UnboundedReceiver, time::Instant};

use super::SyncLeaderTxOrderer;
use crate::{
    rpc::{
        cluster::{GetOrderCommitmentInfo, GetOrderCommitmentInfoResponse},
        prelude::*,
    },
    task::{send_transaction_list_to_mev_searcher, MevTargetTransaction},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionList {
    pub leader_change_message: LeaderChangeMessage,
    pub rollup_signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderChangeMessage {
    pub rollup_id: RollupId,
    pub executor_address: Address,
    pub platform_block_height: u64,

    pub current_leader_tx_orderer_address: Address,
    pub next_leader_tx_orderer_address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignMessage {
    pub rollup_id: RollupId,
    pub executor_address: String,
    pub platform_block_height: u64,

    pub current_leader_tx_orderer_address: Address,
    pub next_leader_tx_orderer_address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionListResponse {
    pub raw_transaction_list: Vec<String>,
}

impl RpcParameter<AppState> for GetRawTransactionList {
    type Response = GetRawTransactionListResponse;

    fn method() -> &'static str {
        "get_raw_transaction_list"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let start_get_raw_transaction_list_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        let mut raw_transaction_list = Vec::new();

        let rollup_id = self.leader_change_message.rollup_id.clone();

        let rollup_metadata = match RollupMetadata::get(&rollup_id) {
            Ok(metadata) => metadata,
            Err(err) => {
                tracing::error!(
                    "Failed to get rollup metadata - rollup_id: {:?} / error: {:?}",
                    rollup_id,
                    err,
                );

                return Ok(GetRawTransactionListResponse {
                    raw_transaction_list: Vec::new(),
                });
            }
        };

        let rollup = Rollup::get(&rollup_id)?;

        let start_batch_number = rollup_metadata.provided_batch_number;
        let mut current_provided_batch_number = start_batch_number;
        let mut current_provided_transaction_order = rollup_metadata.provided_transaction_order;

        while let Ok(batch) = Batch::get(&rollup_id, current_provided_batch_number) {
            let start_transaction_order = if current_provided_batch_number == start_batch_number {
                current_provided_transaction_order + 1
            } else {
                0
            };

            raw_transaction_list.extend(extract_raw_transactions(
                batch,
                start_transaction_order as u64,
            ));

            current_provided_batch_number += 1;
            current_provided_transaction_order = -1;
        }

        if let Ok(can_provide_transaction_info) = CanProvideTransactionInfo::get(&rollup_id) {
            if let Some(can_provide_transaction_orderers) = can_provide_transaction_info
                .can_provide_transaction_orders_per_batch
                .get(&current_provided_batch_number)
            {
                let valid_end_transaction_order = get_last_valid_transaction_order(
                    can_provide_transaction_orderers,
                    current_provided_transaction_order,
                );

                fetch_and_append_transactions(
                    &rollup_id,
                    current_provided_batch_number,
                    (current_provided_transaction_order + 1) as u64,
                    valid_end_transaction_order,
                    &mut raw_transaction_list,
                )?;

                current_provided_transaction_order = valid_end_transaction_order;

                if current_provided_transaction_order
                    == rollup.max_transaction_count_per_batch as i64 - 1
                {
                    current_provided_batch_number += 1;
                    current_provided_transaction_order = -1;
                }
            }
        }

        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            self.leader_change_message.platform_block_height,
        )?;

        let mut mut_rollup_metadata = RollupMetadata::get_mut(&rollup_id)?;

        let mut batch_number_list_to_delete = Vec::new();
        for batch_number in start_batch_number..current_provided_batch_number {
            batch_number_list_to_delete.push(batch_number);
        }

        mut_rollup_metadata.provided_batch_number = current_provided_batch_number;
        mut_rollup_metadata.provided_transaction_order = current_provided_transaction_order;

        let leader_tx_orderer_rpc_info = cluster
            .get_tx_orderer_rpc_info(&self.leader_change_message.next_leader_tx_orderer_address)
            .ok_or_else(|| {
                tracing::error!(
                    "TxOrderer RPC info not found for address {:?}",
                    self.leader_change_message.next_leader_tx_orderer_address
                );
                Error::TxOrdererInfoNotFound
            })?;

        let signer = context.get_signer(rollup.platform).await.map_err(|_| {
            tracing::error!("Signer not found for platform {:?}", rollup.platform);
            Error::SignerNotFound
        })?;

        let tx_orderer_address = signer.address().clone();

        let is_next_leader =
            tx_orderer_address == self.leader_change_message.next_leader_tx_orderer_address;

        let mut mut_cluster_metadata = ClusterMetadata::get_mut(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        if mut_cluster_metadata.is_leader == false {
            if let Some(current_leader_tx_orderer_rpc_info) =
                mut_cluster_metadata.leader_tx_orderer_rpc_info.clone()
            {
                let current_leader_tx_orderer_cluster_rpc_url = current_leader_tx_orderer_rpc_info
                    .cluster_rpc_url
                    .clone()
                    .unwrap();

                let parameter = GetOrderCommitmentInfo {
                    rollup_id: self.leader_change_message.rollup_id.clone(),
                };

                match context
                    .rpc_client()
                    .request_with_priority::<&GetOrderCommitmentInfo, GetOrderCommitmentInfoResponse>(
                        current_leader_tx_orderer_cluster_rpc_url.clone(),
                        GetOrderCommitmentInfo::method(),
                        &parameter,
                        Id::Null,
                        Priority::High,
                    )
                    .await
                {
                    Ok(response) => {

                      tracing::info!(
                          "Get order commitment info - current leader external rpc response: {:?}",
                          response
                      );

                      mut_rollup_metadata.batch_number = response.batch_number;
                      mut_rollup_metadata.transaction_order = response.transaction_order;
                    }
                    Err(error) => {
                        tracing::error!(
                            "Get order commitment info - current leader external rpc error: {:?}",
                            error
                        );
                    }
                }
            } else {
                tracing::warn!(
                    "Current leader tx orderer RPC info not found for address {:?}",
                    self.leader_change_message.current_leader_tx_orderer_address
                );
            }
        }

        mut_cluster_metadata.platform_block_height =
            self.leader_change_message.platform_block_height;
        mut_cluster_metadata.is_leader = is_next_leader;
        mut_cluster_metadata.leader_tx_orderer_rpc_info = Some(leader_tx_orderer_rpc_info.clone());

        let signer = context.get_signer(rollup.platform).await?;
        let current_tx_orderer_address = signer.address();

        sync_leader_tx_orderer(
            context.clone(),
            cluster,
            current_tx_orderer_address,
            self.leader_change_message.clone(),
            self.rollup_signature,
            mut_rollup_metadata.batch_number,
            mut_rollup_metadata.transaction_order,
            mut_rollup_metadata.provided_batch_number,
            mut_rollup_metadata.provided_transaction_order,
        )
        .await;

        mut_cluster_metadata.update()?;
        let _ = mut_rollup_metadata.update().map_err(|error| {
            tracing::error!(
                "rollup_metadata update error - rollup id: {:?}, error: {:?}",
                self.leader_change_message.rollup_id,
                error
            );
        });

        let end_get_raw_transaction_list_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        tracing::info!(
            "get_raw_transaction_list - total take time: {:?}",
            end_get_raw_transaction_list_time - start_get_raw_transaction_list_time
        );

        let shared_channel_infos = context.shared_channel_infos();
        let mev_searcher_infos = MevSearcherInfos::get_or(MevSearcherInfos::default).unwrap();

        send_transaction_list_to_mev_searcher(
            &rollup_id,
            raw_transaction_list.clone(),
            shared_channel_infos,
            &mev_searcher_infos,
        );

        let ip_list = mev_searcher_infos.get_ip_list_by_rollup_id(&rollup_id);
        let receivers: Vec<Arc<tokio::sync::Mutex<UnboundedReceiver<MevTargetTransaction>>>> = {
            let map = shared_channel_infos.lock().unwrap();
            ip_list
                .iter()
                .filter_map(|ip| map.get(ip).map(|(_, rx)| Arc::clone(rx)))
                .collect()
        };

        let collected_mev_target_transaction = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let mut sub_tasks = vec![];

        for receiver in receivers {
            let collected_clone = Arc::clone(&collected_mev_target_transaction);
            let rx = Arc::clone(&receiver);

            let sub_task = tokio::spawn(async move {
                let deadline = Instant::now() + Duration::from_millis(5000);

                tokio::select! {
                    _ = tokio::time::sleep_until(deadline) => {}
                    maybe_mev_target_transaction = async {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    } => {
                        if let Some(mev_target_transaction) = maybe_mev_target_transaction {
                            println!("Received mev target transaction: {:?}", mev_target_transaction);
                            collected_clone.lock().await.push(mev_target_transaction);
                        }
                    }
                }
            });

            sub_tasks.push(sub_task);
        }

        let _ = futures::future::join_all(sub_tasks).await;

        {
            let result = collected_mev_target_transaction.lock().await;
            println!("Collected mev target transactions: {:?}", *result);

            for mev_target_transaction in result.iter() {
                raw_transaction_list
                    .extend(mev_target_transaction.backrunning_transaction_list.clone());
            }
        }

        Ok(GetRawTransactionListResponse {
            raw_transaction_list,
        })
    }
}

pub async fn sync_leader_tx_orderer(
    context: AppState,
    cluster: Cluster,
    current_tx_orderer_address: &Address,
    leader_change_message: LeaderChangeMessage,
    rollup_signature: Signature,
    batch_number: u64,
    transaction_order: u64,
    provided_batch_number: u64,
    provided_transaction_order: i64,
) {
    let mut other_cluster_rpc_url_list = cluster.get_other_cluster_rpc_url_list();
    if other_cluster_rpc_url_list.is_empty() {
        tracing::info!("No cluster RPC URLs available for synchronization");
        return;
    }

    if let Some(next_leader_tx_orderer_rpc_info) =
        cluster.get_tx_orderer_rpc_info(&leader_change_message.next_leader_tx_orderer_address)
    {
        let next_leader_tx_orderer_cluster_rpc_url = next_leader_tx_orderer_rpc_info
            .cluster_rpc_url
            .clone()
            .unwrap();

        // Filter out the next leader's cluster URL from the list
        other_cluster_rpc_url_list = other_cluster_rpc_url_list
            .into_iter()
            .filter(|rpc_url| rpc_url != &next_leader_tx_orderer_cluster_rpc_url)
            .collect();

        let parameter = SyncLeaderTxOrderer {
            leader_change_message,
            rollup_signature,
            batch_number,
            transaction_order,
            provided_batch_number,
            provided_transaction_order,
        };

        if next_leader_tx_orderer_rpc_info.tx_orderer_address != current_tx_orderer_address {
            // Directly request the next leader tx_orderer to sync
            let start_sync_leader_tx_order_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos();

            let _result: Result<(), radius_sdk::json_rpc::client::RpcClientError> = context
                .rpc_client()
                .request_with_priority(
                    next_leader_tx_orderer_cluster_rpc_url.clone(),
                    SyncLeaderTxOrderer::method(),
                    &parameter,
                    Id::Null,
                    Priority::High,
                )
                .await;

            let end_sync_leader_tx_order_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos();

            tracing::info!(
                "SyncLeaderTxOrderer - start: {:?} / end: {:?} / gap: {:?} / next_leader_tx_orderer_cluster_rpc_url: {:?}, parameter: {:?}",
                start_sync_leader_tx_order_time,
                end_sync_leader_tx_order_time,
                end_sync_leader_tx_order_time - start_sync_leader_tx_order_time,
                next_leader_tx_orderer_cluster_rpc_url,
                parameter
            );

            // Fire and forget to the rest of the cluster nodes asynchronously
            let urls = other_cluster_rpc_url_list.clone();
            tokio::spawn(async move {
                let _ = context
                    .rpc_client()
                    .fire_and_forget_multicast(
                        urls,
                        SyncLeaderTxOrderer::method(),
                        &parameter,
                        Id::Null,
                    )
                    .await;
            });
        }
    } else {
        tracing::error!(
            "Next leader tx orderer RPC info not found for address {:?}",
            leader_change_message.next_leader_tx_orderer_address
        );
    }
}

fn extract_raw_transactions(batch: Batch, start_transaction_order: u64) -> Vec<String> {
    batch
        .raw_transaction_list
        .into_iter()
        .enumerate()
        .filter_map(|(i, transaction)| {
            if (i as u64) >= start_transaction_order {
                Some(match transaction {
                    RawTransaction::Eth(EthRawTransaction(data)) => data,
                    RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
                })
            } else {
                None
            }
        })
        .collect()
}

fn get_last_valid_transaction_order(
    can_provide_transaction_orders: &BTreeSet<u64>,
    provided_transaction_order: i64,
) -> i64 {
    let mut last_valid_transaction_order = provided_transaction_order;

    for &transaction_order in can_provide_transaction_orders {
        let transaction_order = transaction_order as i64;

        if transaction_order == last_valid_transaction_order + 1 {
            last_valid_transaction_order += 1;
        } else if transaction_order > last_valid_transaction_order {
            break;
        }
    }

    last_valid_transaction_order as i64
}

fn fetch_and_append_transactions(
    rollup_id: &RollupId,
    batch_number: u64,
    start_transaction_order: u64,
    last_valid_transaction_order: i64,
    raw_transaction_list: &mut Vec<String>,
) -> Result<(), RpcError> {
    if last_valid_transaction_order < start_transaction_order as i64 {
        return Ok(());
    }

    for transaction_order in
        start_transaction_order..=last_valid_transaction_order.try_into().unwrap()
    {
        let (raw_transaction, _) =
            RawTransactionModel::get(rollup_id, batch_number, transaction_order)?;
        let raw_transaction = match raw_transaction {
            RawTransaction::Eth(EthRawTransaction(data)) => data,
            RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
        };
        raw_transaction_list.push(raw_transaction);
    }
    Ok(())
}
