use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister;

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);

    pub async fn handler(_parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        // TODO: context.config().cluster_id()
        context.ssal_client().deregister_sequencer("").await?;

        shutdown::init(context.ssal_client());

        Ok(())
    }
}
