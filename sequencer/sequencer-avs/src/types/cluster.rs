use std::collections::HashMap;

use json_rpc::RpcClient;
use tokio::sync::Mutex;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    pub ssal_block_number: u64,
    pub rollup_block_number: u64,
    pub transaction_order: u64,
    pub is_leader: bool,
}

impl ClusterMetadata {
    const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(database: &Database) -> Result<Self, database::Error> {
        database.get(&Self::ID)
    }

    pub fn get_mut(database: &Database) -> Result<Lock<Self>, database::Error> {
        database.get_mut(&Self::ID)
    }

    pub fn put(&self, database: &Database) -> Result<(), database::Error> {
        database.put(&Self::ID, self)
    }

    pub fn new(ssal_block_number: u64, rollup_block_number: u64, transaction_order: u64) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            transaction_order,
        }
    }

    pub fn issue_order_commitment(&mut self) -> OrderCommitment {
        self.transaction_order += 1;
        OrderCommitment::new(self.rollup_block_number, self.transaction_order)
    }
}

pub struct Cluster {
    sequencer_map: Mutex<HashMap<Address, RpcClient>>,
}
