use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister;

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);

    pub async fn handler(_parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let block_number_at_request = context.ssal_client().get_block_number().await?;
        shutdown::init(context.ssal_client().address(), block_number_at_request);

        Ok(())
    }
}
