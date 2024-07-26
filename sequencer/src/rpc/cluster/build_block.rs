use crate::{models::ClusterMetadataModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    pub rollup_id: RollupId,
    pub ssal_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
}

impl BuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(BuildBlock);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadataModel::get_mut() {
            Ok(mut cluster_metadata) => {
                let finalized_block_height = cluster_metadata.rollup_block_height;
                let transaction_count = cluster_metadata.transaction_order;

                let cluster = cluster_metadata
                    .update(
                        context.ssal_client().address(),
                        context.config().cluster_id(),
                        parameter.ssal_block_height,
                        parameter.rollup_block_height,
                    )
                    .await?;
                context.update_cluster(cluster.clone()).await;
                cluster_metadata.commit()?;

                syncer::sync_build_block(
                    cluster.clone(),
                    parameter.rollup_id,
                    parameter.ssal_block_height,
                    parameter.rollup_block_height,
                    transaction_count,
                );

                builder::build_block(
                    context.ssal_client(),
                    cluster,
                    parameter.rollup_id,
                    finalized_block_height,
                    transaction_count,
                    true,
                );

                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    let previous_block_length = 0;
                    let mut cluster_metadata = ClusterMetadataModel::default();

                    let cluster = cluster_metadata
                        .update(
                            context.ssal_client().address(),
                            context.config().cluster_id(),
                            parameter.ssal_block_height,
                            parameter.rollup_block_height,
                        )
                        .await?;
                    context.update_cluster(cluster.clone()).await;
                    cluster_metadata.put()?;

                    syncer::sync_build_block(
                        cluster,
                        parameter.rollup_id,
                        parameter.ssal_block_height,
                        parameter.rollup_block_height,
                        previous_block_length,
                    );

                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
