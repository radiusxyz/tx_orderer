use crate::types::*;

pub fn init(rollup_block_number: RollupBlockNumber) {
    let previous_rollup_block_number = rollup_block_number - 1;
    let previous_block_height: u64 = match BlockMetadata::get(previous_rollup_block_number).ok() {
        Some(block_metadata) => block_metadata.block_height(),
        None => 0,
    };

    tokio::spawn(async move {
        let mut block = Vec::<ProcessedTransaction>::with_capacity(previous_block_height as usize);
        for transaction_order in 0..previous_block_height {
            let processed_transaction =
                ProcessedTransaction::get(rollup_block_number, transaction_order).unwrap();
            block.push(processed_transaction);
        }
        Block::new(block).put(previous_rollup_block_number);
    });
}
