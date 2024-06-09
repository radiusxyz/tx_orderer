use crate::types::*;

pub fn init(
    rollup_block_number: RollupBlockNumber,
    register_block_commitment: bool,
    previous_block_height: u64,
) {
    tokio::spawn(async move {
        let previous_rollup_block_number = rollup_block_number - 1;
        let mut block = Block::new(previous_block_height as usize);
        for transaction_order in 0..previous_block_height {
            let transaction =
                Transaction::get(previous_rollup_block_number, transaction_order).unwrap();
            block.push(transaction);
        }
        block.put(previous_rollup_block_number).unwrap();

        // TODO: Change the seed to getting it from the contract.
        let block_commitment = block.commitment([0; 32]);
        block_commitment.put(previous_rollup_block_number).unwrap();

        // TODO: Implement register_block_commitment() in the contract.
        if register_block_commitment {}
    });
}
