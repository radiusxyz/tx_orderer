use crate::types::*;

pub fn init(
    rollup_block_number: RollupBlockNumber,
    block_height: u64,
    register_block_commitment: bool,
) {
    tokio::spawn(async move {
        let mut block = Block::new(block_height as usize);
        for transaction_order in 0..block_height {
            let transaction = Transaction::get(rollup_block_number, transaction_order).unwrap();
            block.push(transaction);
        }
        block.put(rollup_block_number).unwrap();

        // TODO: Change the seed to getting it from the contract.
        let block_commitment = block.commitment([0; 32]);
        block_commitment.put(rollup_block_number).unwrap();

        // TODO: Implement register_block_commitment() in the contract.
        if register_block_commitment {}
    });
}
