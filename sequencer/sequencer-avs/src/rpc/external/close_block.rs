use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CloseBlock {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for CloseBlock {
    type Response = SequencerStatus;

    fn method_name() -> &'static str {
        stringify!(CloseBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => {
                // Todo: Spawn a syncer task to forward the request to other sequencers.
                // tokio::spawn(async move {});
                self.update_cluster_metadata()?;
                Ok(SequencerStatus::BlockBuildingInProgress)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    self.update_cluster_metadata()?;
                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}

impl CloseBlock {
    /// After the first updating cluster metadata, the sequencer will no longer return
    /// `SequencerStatus::Uninitialized` to both users and rollups.
    fn update_cluster_metadata(self) -> Result<(), RpcError> {
        let next_rollup_block_number = self.rollup_block_number + 1;

        // Create a new block metadata.
        let block_metadata = BlockMetadata::default();
        block_metadata.put(next_rollup_block_number)?;

        // Get the sequencer list corresponding to SSAL block number.
        let sequencer_list = SequencerList::get(self.ssal_block_number)?;

        // Create a new current cluster metadata.
        let cluster_metadata = ClusterMetadata::new(
            self.ssal_block_number,
            next_rollup_block_number,
            sequencer_list,
        );
        cluster_metadata.put()?;
        Ok(())
    }
}
