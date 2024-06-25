use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub ssal_block_number: u64,
    pub rollup_block_number: u64,
    pub previous_block_height: u64,
}

impl SyncBuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(SyncBuildBlock);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;
        tracing::info!("{}: {:?}", Self::METHOD_NAME, parameter);

        match ClusterMetadata::get_mut() {
            Ok(cluster_metadata) => {
                // TODO:
                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    // TODO:
                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
