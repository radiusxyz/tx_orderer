use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {}

#[async_trait]
impl RpcMethod for SendTransaction {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SendTransaction)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        Ok(())
    }
}
