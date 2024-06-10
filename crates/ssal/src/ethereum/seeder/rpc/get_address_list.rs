use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAddressList {
    pub sequencer_list: Vec<PublicKey>,
}

impl RpcMethod for GetAddressList {
    type Response = Vec<Option<RpcAddress>>;

    fn method_name() -> &'static str {
        stringify!(GetAddressList)
    }
}
