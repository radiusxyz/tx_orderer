use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub rollup_id: RollupId,
    pub ssal_block_height: u64,
    pub rollup_block_height: u64,
    pub previous_block_length: u64,
}

impl SyncBuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadata::get_mut() {
            Ok(mut cluster_metadata) => {
                let previous_rollup_block_height = cluster_metadata.rollup_block_height;

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

                builder::build_block(
                    context.ssal_client(),
                    cluster,
                    parameter.rollup_id,
                    previous_rollup_block_height,
                    parameter.previous_block_length,
                    false,
                );

                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    let mut cluster_metadata = ClusterMetadata::default();
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

                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
