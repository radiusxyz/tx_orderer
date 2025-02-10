use std::{collections::HashMap, sync::Arc};

use radius_sdk::json_rpc::client::RpcClient;
use tokio::sync::Mutex;

use crate::{error::Error, types::*, util::fetch_raw_transaction_info};

pub struct MerkleTreeManager {
    inner: Arc<Mutex<HashMap<String, MerkleTree>>>,
}

impl Clone for MerkleTreeManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for MerkleTreeManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::default())),
        }
    }
}

impl MerkleTreeManager {
    pub async fn init(rpc_client: &RpcClient) -> Self {
        let merkle_tree_manager = Self::default();

        let rollup_id_list = RollupIdList::get_or(RollupIdList::default).unwrap();
        for rollup_id in rollup_id_list.iter() {
            let merkle_tree = MerkleTree::new();

            if let Some(rollup_metadata) = RollupMetadata::get(rollup_id).ok() {
                if rollup_metadata.transaction_order > 0 {
                    tracing::info!(
                        "Building merkle tree for rollup - rollup_id: {:?} / rollup_block_height: {:?} / transaction_order: {:?}",
                        rollup_id,
                        rollup_metadata.rollup_block_height,
                        rollup_metadata.transaction_order
                    );
                    let rollup = Rollup::get(rollup_id).unwrap();
                    let latest_cluster_block_height = LatestClusterBlockHeight::get_or(
                        rollup.platform,
                        rollup.service_provider,
                        &rollup.cluster_id,
                        LatestClusterBlockHeight::default,
                    )
                    .unwrap();

                    let cluster = Cluster::get(
                        rollup.platform,
                        rollup.service_provider,
                        &rollup.cluster_id,
                        latest_cluster_block_height.get_block_height(),
                    )
                    .unwrap();

                    for index in 0..rollup_metadata.transaction_order {
                        let get_raw_transaction_result = RawTransactionModel::get(
                            rollup_id,
                            rollup_metadata.rollup_block_height,
                            index,
                        );

                        let raw_transaction_hash = match get_raw_transaction_result {
                            Ok((raw_transaction, _)) => raw_transaction.raw_transaction_hash(),
                            Err(_) => {
                                tracing::warn!(
                                "Failed to get raw transaction - rollup_id: {:?} / rollup_block_height: {:?} / index: {:?}",
                                rollup_id,
                                rollup_metadata.rollup_block_height,
                                index
                            );

                                let raw_transaction_hash = match fetch_raw_transaction_info(
                                    rpc_client,
                                    &cluster,
                                    &rollup_id,
                                    rollup_metadata.rollup_block_height,
                                    index,
                                )
                                .await
                                {
                                    Ok((raw_transaction, _)) => {
                                        raw_transaction.raw_transaction_hash()
                                    }
                                    Err(error) => {
                                        tracing::warn!(
                                        "Failed to fetch raw transaction - rollup_id: {:?} / rollup_block_height: {:?} / index: {:?} / error: {:?}",
                                        rollup_id,
                                        rollup_metadata.rollup_block_height,
                                        index,
                                        error
                                    );

                                        let encrypted_transaction = EncryptedTransactionModel::get(
                                            rollup_id,
                                            rollup_metadata.rollup_block_height,
                                            index,
                                        )
                                        .unwrap();

                                        encrypted_transaction.raw_transaction_hash()
                                    }
                                };

                                raw_transaction_hash
                            }
                        };

                        merkle_tree.add_data(raw_transaction_hash.as_ref()).await;
                    }
                }
            }

            merkle_tree_manager.insert(rollup_id, merkle_tree).await;
        }

        merkle_tree_manager
    }

    pub async fn insert(&self, rollup_id: &str, merkle_tree: MerkleTree) {
        let mut lock = self.inner.lock().await;
        lock.insert(rollup_id.to_owned(), merkle_tree);
    }

    pub async fn get(&self, rollup_id: &str) -> Result<MerkleTree, Error> {
        let lock = self.inner.lock().await;
        let merkle_tree = lock
            .get(rollup_id)
            .ok_or(Error::MerkleTreeDoesNotExist(rollup_id.to_owned()))?;

        Ok(merkle_tree.clone())
    }
}
