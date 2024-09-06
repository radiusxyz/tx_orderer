use crate::types::*;

pub fn block_builder(rollup_id: &String, rollup_metadata: RollupMetadata) {
    let rollup_id = rollup_id.clone();
    let block_height = rollup_metadata.block_height();
    let transaction_order = rollup_metadata.transaction_order();

    tokio::spawn(async move {
        for order_index in 0..transaction_order {
            let encrypted_transaction =
                EncryptedTransactionModel::get(&rollup_id, block_height, order_index).unwrap();
        }
    });
}

pub fn fetch_missing_transaction(transaction_order: u64) {}
