use std::sync::Arc;

use sha3::{Digest, Keccak256};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct MerkleTree {
    pub nodes: Arc<Mutex<Vec<Vec<[u8; 32]>>>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(vec![vec![]])),
        }
    }

    pub async fn add_data(&self, data: &str) -> (u64, Vec<[u8; 32]>) {
        let mut nodes = self.nodes.lock().await;
        Self::update_tree(&mut nodes);

        let pre_merkle_path = Self::get_pre_merkle_path(&nodes);

        let hashed_data = Self::hash(data.as_bytes());

        nodes[0].push(hashed_data);

        ((nodes[0].len() - 1) as u64, pre_merkle_path)
    }

    fn get_pre_merkle_path(nodes: &Vec<Vec<[u8; 32]>>) -> Vec<[u8; 32]> {
        let mut proof = vec![];
        let mut leaf_node_index: usize = 0;

        let leaf_node_count = nodes[0].len();

        if leaf_node_count == 0 {
            return proof;
        }

        if leaf_node_count == 1 {
            return vec![nodes[0][0]];
        }

        loop {
            let mut current_level = 0;
            let mut target_index = leaf_node_index;

            while nodes[current_level].len() > target_index + 1 {
                current_level += 1;
                target_index /= 2;
            }

            proof.push(nodes[current_level][target_index]);

            leaf_node_index += 2_usize.pow(current_level as u32);

            if leaf_node_index >= leaf_node_count {
                break;
            }
        }

        proof
    }

    fn update_tree(nodes: &mut Vec<Vec<[u8; 32]>>) {
        let mut current_level = 0;

        if nodes[current_level].is_empty() {
            return;
        }

        while nodes[current_level].len() % 2 == 0 {
            let level = &nodes[current_level];
            let len = level.len();
            let right_node = &level[len - 1];
            let left_node = &level[len - 2];

            let parent_node = Self::hash(&Self::concat_arrays(*left_node, *right_node));

            if nodes.len() <= current_level + 1 {
                nodes.push(vec![parent_node]);
            } else {
                nodes[current_level + 1].push(parent_node);
            }

            current_level += 1;
        }
    }

    fn concat_arrays(a: [u8; 32], b: [u8; 32]) -> [u8; 64] {
        let mut array: [u8; 64] = [0; 64];
        for (index, value) in a.into_iter().chain(b.into_iter()).enumerate() {
            array[index] = value;
        }
        array
    }

    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub async fn finalize_tree(&self) {
        let mut nodes = self.nodes.lock().await;

        let leaves = nodes[0].clone();
        *nodes = vec![leaves];

        let mut current_level = 0;

        while nodes[current_level].len() > 1 {
            let mut next_level = vec![];
            let level = &nodes[current_level];

            let mut i = 0;
            while i < level.len() {
                let left = level[i];
                let right = if i + 1 < level.len() {
                    level[i + 1]
                } else {
                    level[i]
                };

                let parent = Self::hash(&Self::concat_arrays(left, right));
                next_level.push(parent);
                i += 2;
            }

            nodes.push(next_level);
            current_level += 1;
        }
    }

    pub async fn verify_proof(
        &self,
        mut pre_merkle_path: Vec<[u8; 32]>,
        mut post_merkle_path: Vec<[u8; 32]>,
        mut index: usize,
        data: &str,
        merkle_root: [u8; 32],
    ) -> bool {
        let mut current_hash = Self::hash(data.as_bytes());

        pre_merkle_path.reverse();

        while !pre_merkle_path.is_empty() || !post_merkle_path.is_empty() {
            let sibling = if index % 2 == 0 {
                if post_merkle_path.is_empty() {
                    return false;
                }
                post_merkle_path.remove(0)
            } else {
                if pre_merkle_path.is_empty() {
                    return false;
                }
                pre_merkle_path.remove(0)
            };

            if index % 2 == 0 {
                current_hash = Self::hash(&Self::concat_arrays(current_hash, sibling));
            } else {
                current_hash = Self::hash(&Self::concat_arrays(sibling, current_hash));
            }

            index /= 2;
        }

        current_hash == merkle_root
    }

    pub async fn get_merkle_root(&self) -> [u8; 32] {
        let nodes = self.nodes.lock().await;
        if nodes[0].is_empty() {
            return Self::hash(b"");
        }

        nodes
            .last()
            .and_then(|level| level.get(0).cloned())
            .unwrap()
    }

    pub async fn get_post_merkle_path(&self, mut index: usize) -> Vec<[u8; 32]> {
        let nodes = self.nodes.lock().await;
        let mut post_merkle_path = Vec::new();

        if nodes[0].len() <= index {
            return post_merkle_path;
        }

        for level in nodes.iter().take(nodes.len() - 1) {
            if index % 2 == 0 {
                post_merkle_path.push(level[index]);
            }
            index /= 2;
        }

        post_merkle_path
    }

    pub async fn get_all_nodes_hex_string(&self) -> Vec<Vec<String>> {
        let locked = self.nodes.lock().await;
        locked
            .iter()
            .map(|level| {
                level
                    .iter()
                    .map(|hash| const_hex::encode_prefixed(hash))
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_post_merkle_path() {
        for i in 2..=16 {
            let tree = MerkleTree::new();
            let mut transaction_order_list = Vec::new();
            let mut pre_merkle_path_list = Vec::new();

            let (transaction_order, pre_merkle_path) = tree
                .add_data("0x83b002caeea5a70ec6b94fd2cf71de5321fd3b94e7ce4535aea3028e31f3b10d")
                .await;
            transaction_order_list.push(transaction_order);
            pre_merkle_path_list.push(pre_merkle_path);

            for _j in 1..i {
                let (transaction_order, pre_merkle_path) = tree
                    .add_data("0x83b002caeea5a70ec6b94fd2cf71de5321fd3b94e7ce4535aea3028e31f3b10d")
                    .await;

                transaction_order_list.push(transaction_order);
                pre_merkle_path_list.push(pre_merkle_path);
            }

            tree.finalize_tree().await;

            for _j in 0..i {
                let transaction_order = transaction_order_list.pop().unwrap();
                let pre_merkle_path = pre_merkle_path_list.pop().unwrap();
                let post_merkle_path = tree.get_post_merkle_path(transaction_order as usize).await;

                let is_valid = tree
                    .verify_proof(
                        pre_merkle_path.clone(),
                        post_merkle_path.clone(),
                        transaction_order as usize,
                        "0x83b002caeea5a70ec6b94fd2cf71de5321fd3b94e7ce4535aea3028e31f3b10d",
                        tree.get_merkle_root().await,
                    )
                    .await;

                if !is_valid {
                    let all_nodes = tree.get_all_nodes_hex_string().await;
                    println!("{:?}", all_nodes);

                    println!(
                        "total_nodes: {} /transaction_order: {:?}",
                        i, transaction_order
                    );
                    println!(
                        "pre_merkle_path: {:?}",
                        pre_merkle_path
                            .iter()
                            .map(|x| const_hex::encode_prefixed(x))
                            .collect::<Vec<String>>()
                    );
                    println!(
                        "post_merkle_path: {:?}",
                        post_merkle_path
                            .iter()
                            .map(|x| const_hex::encode_prefixed(x))
                            .collect::<Vec<String>>()
                    );

                    return;
                }
            }
        }
    }
}
