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
        tracing::info!("{}: {:?}", Self::METHOD_NAME, parameter);
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadata::get_mut() {
            Ok(mut cluster_metadata) => {
                let previous_rollup_block_number = cluster_metadata.rollup_block_number;
                let previous_block_height = cluster_metadata.transaction_order;

                let cluster = cluster_metadata
                    .update(
                        context.ssal_client().address(),
                        context.config().cluster_id(),
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                    )
                    .await?;
                context.update_cluster(cluster.clone()).await;
                cluster_metadata.commit()?;

                syncer::sync_build_block(
                    cluster.clone(),
                    parameter.ssal_block_number,
                    parameter.rollup_block_number,
                    previous_block_height,
                );

                builder::build_block(
                    context.ssal_client(),
                    cluster,
                    previous_rollup_block_number,
                    previous_block_height,
                    true,
                );

                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    let previous_block_height = 0;
                    let mut cluster_metadata = ClusterMetadata::default();

                    let cluster = cluster_metadata
                        .update(
                            context.ssal_client().address(),
                            context.config().cluster_id(),
                            parameter.ssal_block_number,
                            parameter.rollup_block_number,
                        )
                        .await?;
                    context.update_cluster(cluster.clone()).await;
                    cluster_metadata.put()?;

                    syncer::sync_build_block(
                        cluster,
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                        previous_block_height,
                    );

                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
