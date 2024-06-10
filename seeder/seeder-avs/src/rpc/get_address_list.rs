use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAddressList {
    pub sequencer_list: Vec<PublicKey>,
}

#[async_trait]
impl RpcMethod for GetAddressList {
    type Response = Vec<Option<RpcAddress>>;

    fn method_name() -> &'static str {
        stringify!(GetAddressList)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        let sequencer_list: Vec<Option<RpcAddress>> = self
            .sequencer_list
            .iter()
            .map(|sequencer_public_key| {
                database()
                    .get::<PublicKey, RpcAddress>(sequencer_public_key)
                    .ok()
            })
            .collect();
        Ok(sequencer_list)
    }
}
