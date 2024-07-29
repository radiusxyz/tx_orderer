use crate::{models::ClusterMetadataModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBlock {
    pub rollup_id: RollupId,
    pub ssal_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

impl SyncBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster = context.cluster().await?;

        match ClusterMetadataModel::get_mut() {
            Ok(mut cluster_metadata_model) => {
                builder::build_block(
                    context.ssal_client(),
                    cluster,
                    parameter.rollup_id,
                    parameter.rollup_block_height,
                    parameter.transaction_order,
                    false,
                );

                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    let cluster_metadata = ClusterMetadataModel::new(
                        parameter.ssal_block_height,
                        parameter.rollup_block_height,
                        parameter.transaction_order,
                        false, // TODO: check
                    );

                    cluster_metadata.put()?;

                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
