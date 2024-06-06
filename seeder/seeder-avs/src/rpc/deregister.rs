use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub public_key: PublicKey,
}

#[async_trait]
impl RpcMethod for Deregister {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(Deregister)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        database()
            .delete(&self.public_key)
            .map_err(|error| error.into())
    }
}
