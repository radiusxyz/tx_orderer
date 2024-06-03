use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {}

#[async_trait]
impl RpcMethod for SyncTransaction {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SyncTransaction)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        Ok(())
    }
}
