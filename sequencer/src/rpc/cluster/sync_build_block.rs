use crate::{models::ClusterMetadataModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub rollup_id: RollupId,
    pub ssal_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

impl SyncBuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadataModel::get_mut() {
            Ok(mut cluster_metadata_model) => {
                let previous_rollup_block_height = cluster_metadata_model.rollup_block_height;

                let cluster = cluster_metadata_model
                    .update(
                        context.ssal_client().address(),
                        context.config().cluster_id(),
                        parameter.ssal_block_height,
                        parameter.rollup_block_height,
                    )
                    .await?;
                context.update_cluster(cluster.clone()).await;
                cluster_metadata_model.commit()?;

                builder::build_block(
                    context.ssal_client(),
                    cluster,
                    parameter.rollup_id,
                    previous_rollup_block_height,
                    parameter.transaction_order,
                    false,
                );

                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
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

                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
