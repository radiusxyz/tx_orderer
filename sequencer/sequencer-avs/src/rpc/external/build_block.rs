use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
}

impl BuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(BuildBlock);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<SsalClient>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => {
                tracing::info!("{}: {:?}", Self::METHOD_NAME, parameter);
                // After updating the cluster metadata, the previous block height remains unchanged.
                // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                // sync the previous block height.
                update_cluster_metadata(
                    parameter.ssal_block_number,
                    parameter.rollup_block_number,
                )?;

                let previous_block_height =
                    BlockMetadata::get(cluster_metadata.rollup_block_number())?.block_height();

                block_syncer::init(
                    parameter.ssal_block_number,
                    parameter.rollup_block_number,
                    previous_block_height,
                );

                block_builder::init(
                    cluster_metadata.ssal_block_number(),
                    cluster_metadata.rollup_block_number(),
                    previous_block_height,
                    true,
                );

                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    tracing::info!("1");
                    // After updating the cluster metadata, the previous block height remains unchanged.
                    // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                    // sync the previous block height.
                    update_cluster_metadata(
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                    )?;

                    block_syncer::init(
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                        0,
                    );

                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
