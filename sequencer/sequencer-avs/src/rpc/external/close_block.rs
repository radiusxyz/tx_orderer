use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CloseBlock {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for CloseBlock {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(CloseBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        // Get the sequencer list corresponding to SSAL block number.
        let sequencer_list_key = SequencerListKey::new(self.ssal_block_number);
        let sequencer_list: SequencerList = database().get(&sequencer_list_key)?;
        let next_rollup_block_number = self.rollup_block_number + 1;

        // Create a new block and a cluster based on the request.
        let block_key = BlockKey::new(self.ssal_block_number, next_rollup_block_number);
        let block = Block::new();

        let cluster = Cluster::new(
            self.ssal_block_number,
            next_rollup_block_number,
            sequencer_list,
        );
        database().put(&CURRENT_CLUSTER, &cluster)?;
        Ok(())
    }
}
