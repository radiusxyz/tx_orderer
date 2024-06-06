use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub signature: Signature,
    pub public_key: PublicKey,
}

impl RpcMethod for Deregister {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(Deregister)
    }
}
