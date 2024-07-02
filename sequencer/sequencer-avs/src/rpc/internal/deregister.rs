use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister;

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);

    pub async fn handler(_parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        context
            .ssal_client()
            .deregister_sequencer(context.config().cluster_id())
            .await?;

        context.ssal_client().deregister_operator().await?;

        shutdown::init(context.ssal_client());

        Ok(())
    }
}
