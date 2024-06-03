use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CloseBlock {}

#[async_trait]
impl RpcMethod for CloseBlock {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(CloseBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        Ok(())
    }
}
