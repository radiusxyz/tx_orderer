use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub rollup_id: RollupId,
}

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // TODO: context.config().cluster_id()
        context
            .get_rollup_cluster(&parameter.rollup_id)
            .unwrap()
            .get_liveness_client()
            .unwrap()
            .deregister_sequencer(parameter.rollup_id)
            .await?;

        Ok(())
    }
}
