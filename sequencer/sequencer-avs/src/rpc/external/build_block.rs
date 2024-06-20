use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    pub ssal_block_number: u64,
    pub rollup_block_number: u64,
}

impl BuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(BuildBlock);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let database = context.database();

        match ClusterMetadata::get_mut(&database) {
            Ok(cluster_metadata) => {
                tracing::info!("{}: {:?}", Self::METHOD_NAME, parameter);
                // After updating the cluster metadata, the previous block height remains unchanged.
                // Calling `update_cluster_metadata()` before running the syncer makes it safe to
                // sync the previous block height.
                let updated_cluster_metadata = ClusterMetadata::new(
                    parameter.ssal_block_number,
                    parameter.rollup_block_number,
                    cluster_metadata.sequencer_list.clone(),
                    // TODO: select a leader
                    false,
                )
                .put(&database)?;

                let previous_block_height =
                    BlockMetadata::get(&database, cluster_metadata.rollup_block_number())?
                        .block_height();

                syncer::sync_build_block(
                    ssal_block_number,
                    rollup_block_number,
                    previous_block_height,
                    cluster_metadata,
                );

                builder::build_block(
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
                        &database,
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                    )?;

                    syncer::sync_build_block(
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
