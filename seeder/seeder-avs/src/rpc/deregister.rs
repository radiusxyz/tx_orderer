use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub signature: Signature,
    pub public_key: PublicKey,
}

#[async_trait]
impl RpcMethod for Deregister {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(Deregister)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        let signer = self.signature.recover(self.public_key.as_bytes())?;
        if signer == *self.public_key {
            database()
                .delete(&self.public_key)
                .map_err(|error| error.into())
        } else {
            Err(Error::SignatureMismatch.into())
        }
    }
}
