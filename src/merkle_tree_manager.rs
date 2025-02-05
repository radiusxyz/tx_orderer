use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{error::Error, types::*};

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
    pub async fn init() -> Self {
        let merkle_tree_manager = Self::default();

        let rollup_id_list = RollupIdList::get_or(RollupIdList::default).unwrap();
        for rollup_id in rollup_id_list.iter() {
            let merkle_tree = MerkleTree::new();

            if let Some(rollup_metadata) = RollupMetadata::get(rollup_id).ok() {
                tracing::debug!(
                    "Building merkle tree for rollup: {:?} / transaction_order: {:?}",
                    rollup_id,
                    rollup_metadata.transaction_order
                );

                for index in 0..rollup_metadata.transaction_order {
                    let (raw_transaction, _) = RawTransactionModel::get(
                        rollup_id,
                        rollup_metadata.rollup_block_height,
                        index,
                    )
                    .unwrap();
                    merkle_tree
                        .add_data(raw_transaction.raw_transaction_hash().as_ref())
                        .await;
                }
            }

            println!("{:?}", merkle_tree.get_merkle_root().await);
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
