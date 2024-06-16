use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
    pub previous_block_height: u64,
}

impl SyncBuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<()>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => {
                tracing::info!("{}: {:?}", Self::METHOD_NAME, parameter);

                update_cluster_metadata(
                    parameter.ssal_block_number,
                    parameter.rollup_block_number,
                )?;

                block_builder::init(
                    cluster_metadata.ssal_block_number(),
                    cluster_metadata.rollup_block_number(),
                    parameter.previous_block_height,
                    false,
                );

                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    update_cluster_metadata(
                        parameter.ssal_block_number,
                        parameter.rollup_block_number,
                    )?;

                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
