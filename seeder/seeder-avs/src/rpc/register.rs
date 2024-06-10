use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub public_key: PublicKey,
    pub sequencer_rpc_address: RpcAddress,
}

#[async_trait]
impl RpcMethod for Register {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(Register)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        tracing::info!("{:?}", self);
        health_check(&self.sequencer_rpc_address).await?;
        database()
            .put(&self.public_key, &self.sequencer_rpc_address)
            .map_err(|error| error.into())
    }
}
