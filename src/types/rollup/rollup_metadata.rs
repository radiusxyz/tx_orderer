use std::collections::BTreeSet;

use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    pub batch_number: u64,
    pub transaction_order: u64,
    pub max_transaction_order: u64,

    pub max_gas_limit: u64,
    pub current_gas: u64,

    pub cluster_id: String,

    pub provided_batch_number: u64,
    pub provided_transaction_order: i64,

    pub can_provide_batch_number: u64,
    pub can_provide_transaction_orders: BTreeSet<u64>,
}

impl Default for RollupMetadata {
    fn default() -> Self {
        Self {
            batch_number: 0,
            transaction_order: 0,
            max_transaction_order: 5,

            max_gas_limit: 0,
            current_gas: 0,

            cluster_id: String::new(),

            provided_batch_number: 0,
            provided_transaction_order: -1,

            can_provide_batch_number: 0,
            can_provide_transaction_orders: BTreeSet::new(),
        }
    }
}

impl RollupMetadata {
    pub fn is_overflow_gas_limit(&self, transaction_gas_limit: u64) -> bool {
        self.max_gas_limit != 0 && self.current_gas + transaction_gas_limit > self.max_gas_limit
    }
}
