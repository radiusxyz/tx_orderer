use crate::rpc::{external::SyncBuildBlock, prelude::*, util::update_cluster_metadata};

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
            Ok(_cluster_metadata) => {
                tracing::info!("BuildBlock: {:?}", self);

                // After updating the cluster metadata, the previous block height remains unchanged.
                // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                // sync the previous block height.
                update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;

                // Run the syncer.
                block_syncer::init(self.ssal_block_number, self.rollup_block_number);

                // Run the block builder.
                block_builder::init(self.rollup_block_number, true);
                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    // After updating the cluster metadata, the previous block height remains unchanged.
                    // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                    // sync the previous block height.
                    update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;

                    // Skip the block builder and run the syncer because the previous block does not exist.
                    block_syncer::init(self.ssal_block_number, self.rollup_block_number);
                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}

impl BuildBlock {
    fn syncer(&self) -> Result<(), RpcError> {
        let me = Me::get()?;
        let sequencer_list = SequencerList::get(self.ssal_block_number)?;

        let previous_rollup_block_number = self.rollup_block_number - 1;
        let previous_block_height: u64 = match BlockMetadata::get(previous_rollup_block_number).ok()
        {
            Some(block_metadata) => block_metadata.block_height(),
            None => 0,
        };

        let rpc_method = SyncBuildBlock {
            ssal_block_number: self.ssal_block_number,
            rollup_block_number: self.rollup_block_number,
            previous_block_height,
        };

        tokio::spawn(async move {
            for (public_key, rpc_address) in sequencer_list.into_iter() {
                // Always skip forwarding to myself to avoid redundant handling.
                if me == public_key {
                    continue;
                }

                if let Some(rpc_address) = rpc_address {
                    let rpc_method = rpc_method.clone();
                    tokio::spawn(async move {
                        let rpc_client = RpcClient::new(rpc_address, 1).unwrap();
                        let _ = rpc_client.request(rpc_method).await;
                    });
                }
            }
        });

        Ok(())
    }
}
