use std::collections::{BTreeSet, HashMap};

use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

use crate::{error::Error, types::ClusterId};

use super::RollupId;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &RollupId))]
pub struct CanProvideTransactionInfo {
    pub can_provide_transaction_orders_per_batch: HashMap<u64, BTreeSet<u64>>,
}

impl Default for CanProvideTransactionInfo {
    fn default() -> Self {
        Self {
            can_provide_transaction_orders_per_batch: HashMap::new(),
        }
    }
}

impl CanProvideTransactionInfo {
    pub fn remove_can_provide_transaction_orders(
        rollup_id: &RollupId,
        batch_number: u64,
    ) -> Result<(), Error> {
        let mut can_provide_transactions_per_batch = Self::get_mut_or(rollup_id, Self::default)?;

        can_provide_transactions_per_batch
            .can_provide_transaction_orders_per_batch
            .retain(|&key, _| key > batch_number);

        can_provide_transactions_per_batch.update()?;

        Ok(())
    }

    pub fn add_can_provide_transaction_orders(
        rollup_id: &RollupId,
        batch_number: u64,
        transaction_order_list: Vec<u64>,
    ) -> Result<(), Error> {
        let mut can_provide_transactions_per_batch = Self::get_mut_or(rollup_id, Self::default)?;

        let can_provide_transactions = can_provide_transactions_per_batch
            .can_provide_transaction_orders_per_batch
            .entry(batch_number)
            .or_insert_with(BTreeSet::new);

        can_provide_transactions.extend(transaction_order_list);
        can_provide_transactions_per_batch.update()?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &RollupId))]
pub struct RollupMetadata {
    pub batch_number: u64,
    pub transaction_order: u64,
    pub max_transaction_count_per_batch: u64,

    pub cluster_id: ClusterId,

    pub provided_batch_number: u64,
    pub provided_transaction_order: i64,
}

impl Default for RollupMetadata {
    fn default() -> Self {
        Self {
            batch_number: 0,
            transaction_order: 0,
            max_transaction_count_per_batch: 0,

            cluster_id: String::new(),

            provided_batch_number: 0,
            provided_transaction_order: -1,
        }
    }
}

impl RollupMetadata {
    pub fn check_and_update_batch_info(&mut self) -> bool {
        if self.transaction_order == self.max_transaction_count_per_batch {
            self.batch_number += 1;
            self.transaction_order = 0;

            return true;
        }

        false
    }
}
