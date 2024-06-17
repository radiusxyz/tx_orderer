use json_rpc::RpcClient;

use crate::{rpc::external::GetTransaction, types::*};

pub fn init(
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
    block_height: u64,
    register_block_commitment: bool,
) {
    tokio::spawn(async move {
        let sequencer_list = SequencerList::get(ssal_block_number).unwrap();

        // Build block.
        let mut block = RollupBlock::new(block_height as usize);
        for transaction_order in 0..block_height {
            match UserTransaction::get(rollup_block_number, transaction_order) {
                Ok(transaction) => block.push(transaction),
                Err(error) => {
                    if error.kind() == database::ErrorKind::KeyDoesNotExist {
                        // Fetch the missing transaction from other sequencers.
                        let rpc_method = GetTransaction {
                            rollup_block_number,
                            transaction_order,
                        };

                        // Get the first OK response from the sequencer list.
                        // let transaction: UserTransaction = RpcClient::fetch(
                        //     sequencer_list.address(),
                        //     3,
                        //     GetTransaction::METHOD_NAME,
                        //     rpc_method,
                        // )
                        // .await
                        // .unwrap();

                        // block.push(transaction);
                    } else {
                        // Unlikely.
                        tracing::error!("{}", error);
                    }
                }
            }
        }
        block.put(rollup_block_number).unwrap();

        // TODO: Change the seed to getting it from the contract.
        let block_commitment = block.commitment([0; 32]);
        block_commitment.put(rollup_block_number).unwrap();

        // TODO: Implement register_block_commitment() in the contract.
        if register_block_commitment {}
    });
}
