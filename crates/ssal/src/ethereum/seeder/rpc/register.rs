use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub public_key: PublicKey,
    pub sequencer_rpc_address: RpcAddress,
}

impl RpcMethod for Register {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(Register)
    }
}
