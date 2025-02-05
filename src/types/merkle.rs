use std::sync::Arc;

use sha3::{Digest, Keccak256};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct MerkleTree {
    pub nodes: Arc<Mutex<Vec<Vec<[u8; 32]>>>>, // nodes by tree level
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(vec![vec![]])),
        }
    }

    fn update_tree(nodes: &mut Vec<Vec<[u8; 32]>>) {
        let mut current_level = 0;

        if nodes[current_level].is_empty() {
            return;
        }

        while nodes[current_level].len() % 2 == 0 {
            let level = &nodes[current_level];
            let right_node = &level[level.len() - 1];
            let left_node = &level[level.len() - 2];

            let parent_node = Self::hash(&Self::concat_arrays(*left_node, *right_node));

            if nodes.len() <= current_level + 1 {
                nodes.push(vec![parent_node]);
            } else {
                nodes[current_level + 1].push(parent_node);
            }

            current_level += 1;
        }
    }

    pub async fn finalize_tree(&self) {
        let mut nodes = self.nodes.lock().await;
        let last_node = nodes[0].last().cloned().unwrap_or_default();
        let mut current_level = 0;

        while nodes[current_level].len() > 1 {
            if nodes[current_level].len() % 2 == 1 {
                let left_node = nodes[current_level].last().unwrap();
                let parent_node = Self::hash(&Self::concat_arrays(*left_node, last_node));

                nodes[current_level].push(last_node);
                nodes[current_level + 1].push(parent_node);
            }

            if nodes.len() <= current_level + 1 {
                let left_node = nodes[current_level][0];
                let right_node = nodes[current_level][1];

                let merkle_root = Self::hash(&Self::concat_arrays(left_node, right_node));
                nodes.push(vec![merkle_root]);
            }

            current_level += 1;
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

    pub async fn get_merkle_path(&self, index: usize) -> Vec<[u8; 32]> {
        let nodes = self.nodes.lock().await;
        let mut path = vec![];
        let mut current_index = index;

        for level in nodes.iter() {
            if level.len() <= 1 {
                break;
            }

            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level.len() {
                path.push(level[sibling_index]);
            }

            current_index /= 2;
        }

        path
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

    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub async fn get_post_merkle_path(&self, mut index: usize) -> Vec<[u8; 32]> {
        let nodes = self.nodes.lock().await;
        let mut post_merkle_path = Vec::new();

        for level in nodes.iter().take(nodes.len() - 1) {
            if index % 2 == 0 && index + 1 < level.len() {
                post_merkle_path.push(level[index + 1]);
            }

            index /= 2;
        }

        post_merkle_path
    }

    fn concat_arrays(a: [u8; 32], b: [u8; 32]) -> [u8; 64] {
        let mut array: [u8; 64] = [0; 64];
        for (index, value) in a.into_iter().chain(b.into_iter()).enumerate() {
            array[index] = value;
        }
        array
    }
}
