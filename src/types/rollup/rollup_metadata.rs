use std::collections::BTreeSet;

use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    pub batch_number: u64,
    pub transaction_order: u64,
    pub max_transaction_order: u64,

    pub max_gas_limit: u64,
    pub current_gas: u64,

    pub cluster_id: String,

    pub provided_batch_number: u64,
    pub provided_transaction_order: u64,

    pub can_provide_batch_number: u64,
    pub can_provide_transactions: BTreeSet<u64>,
}
