use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MerkleTree {
    nodes: Vec<Vec<String>>, // nodes by tree level
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![vec![]],
        }
    }

    fn update_tree(&mut self) {
        let mut current_level = 0;

        if self.nodes[current_level].is_empty() {
            return;
        }

        while self.nodes[current_level].len() % 2 == 0 {
            let level = &self.nodes[current_level];
            let right_node = &level[level.len() - 1];
            let left_node = &level[level.len() - 2];

            let parent_node = Self::hash(&(left_node.clone() + right_node));

            if self.nodes.len() <= current_level + 1 {
                self.nodes.push(vec![parent_node]);
            } else {
                self.nodes[current_level + 1].push(parent_node);
            }

            current_level += 1;
        }
    }

    pub fn finalize_tree(&mut self) {
        let last_node = self.nodes[0].last().cloned().unwrap_or_default();
        let mut current_level = 0;

        while self.nodes[current_level].len() > 1 {
            if self.nodes[current_level].len() % 2 == 1 {
                let left_node = self.nodes[current_level].last().unwrap().clone();
                let parent_node = Self::hash(&(left_node.clone() + &last_node));

                self.nodes[current_level].push(last_node.clone());
                self.nodes[current_level + 1].push(parent_node);
            }

            if self.nodes.len() <= current_level + 1 {
                let left_node = self.nodes[current_level][0].clone();
                let right_node = self.nodes[current_level][1].clone();

                let merkle_root = Self::hash(&(left_node.clone() + &right_node));
                self.nodes.push(vec![merkle_root]);
            }

            current_level += 1;
        }
    }

    pub fn add_data(&mut self, data: &str) -> (u64, Vec<String>) {
        self.update_tree();
        println!("Nodes: {:?}", self.nodes);

        let pre_merkle_path = self.get_pre_merkle_path();

        let hashed_data = Self::hash(data);
        self.nodes[0].push(hashed_data);

        ((self.nodes[0].len() - 1) as u64, pre_merkle_path)
    }

    fn get_pre_merkle_path(&self) -> Vec<String> {
        let mut proof = vec![];
        let mut leaf_node_index: usize = 0;

        let leaf_node_count = self.nodes[0].len();

        if leaf_node_count == 0 {
            return proof;
        }

        if leaf_node_count == 1 {
            return vec![self.nodes[0][0].clone()];
        }

        loop {
            let mut current_level = 0;
            let mut target_index = leaf_node_index;

            while self.nodes[current_level].len() > target_index + 1 {
                current_level += 1;
                target_index /= 2;
            }

            proof.push(self.nodes[current_level][target_index].clone());

            leaf_node_index += 2_usize.pow(current_level as u32);

            if leaf_node_index >= leaf_node_count {
                break;
            }
        }

        proof
    }

    pub fn get_merkle_path(&self, index: usize) -> Vec<String> {
        let mut path = vec![];
        let mut current_index = index;

        for level in &self.nodes {
            if level.len() <= 1 {
                break;
            }

            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level.len() {
                path.push(level[sibling_index].clone());
            }

            current_index /= 2;
        }

        path
    }

    pub fn get_merkle_root(&self) -> String {
        if self.nodes[0].is_empty() {
            return Self::hash("");
        }
        self.nodes
            .last()
            .and_then(|level| level.get(0).cloned())
            .unwrap()
    }

    fn hash(data: &str) -> String {
        // let mut hasher = Sha3_256::new();
        let mut hasher = Keccak256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_post_merkle_path(&self, mut index: usize) -> Vec<String> {
        let mut post_merkle_path = Vec::new();

        for level in self.nodes.iter().take(self.nodes.len() - 1) {
            if index % 2 == 0 && index + 1 < level.len() {
                post_merkle_path.push(level[index + 1].clone());
            }

            index /= 2;
        }

        post_merkle_path
    }
}

#[test]
fn block_commitment_test() {
    let mut merkle_tree = MerkleTree::new();

    let data = "Example data";

    loop {
        let (index, pre_merkle_path) = merkle_tree.add_data(data);
        println!("Transaction Index: {}", index);
        println!("Nodes: {:?}", merkle_tree.nodes);
        println!("pre_merkle_path: {:?}", pre_merkle_path);
        println!("");

        if index >= 7 {
            break;
        }
    }

    merkle_tree.finalize_tree();
    println!("Nodes: {:?}", merkle_tree.nodes);
    println!("");

    let merkle_root = merkle_tree.get_merkle_root();
    println!("Merkle root: {:?}", merkle_root);
    println!("");

    let mut index = 0;
    loop {
        let post_merke_path = merkle_tree.get_post_merkle_path(index);
        println!("post_merke_path: {:?}", post_merke_path);

        if index >= 7 {
            break;
        }
        index += 1;
    }
}
