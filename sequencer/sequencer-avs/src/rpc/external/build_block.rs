use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for BuildBlock {
    type Response = SequencerStatus;

    fn method_name() -> &'static str {
        stringify!(BuildBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => {
                tracing::info!("{}: {:?}", Self::method_name(), self);

                // After updating the cluster metadata, the previous block height remains unchanged.
                // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                // sync the previous block height.
                update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                println!("{:?}", cluster_metadata);
                let previous_block_height =
                    BlockMetadata::get(cluster_metadata.rollup_block_number())?.block_height();
                block_syncer::init(
                    self.ssal_block_number,
                    self.rollup_block_number,
                    previous_block_height,
                );
                block_builder::init(
                    cluster_metadata.rollup_block_number(),
                    previous_block_height,
                    true,
                );
                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    // After updating the cluster metadata, the previous block height remains unchanged.
                    // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                    // sync the previous block height.
                    update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                    block_syncer::init(self.ssal_block_number, self.rollup_block_number, 0);
                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
