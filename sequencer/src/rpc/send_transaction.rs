use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct SendEncryptedTransaction {}

#[async_trait]
impl RpcMethod for SendEncryptedTransaction {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SendEncryptedTransaction)
    }

    async fn handler(self, context: Arc<Database>) -> Result<Self::Response, Error> {
        Ok(())
    }
}
