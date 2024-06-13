use serde::{Deserialize, Serialize};

use crate::ethereum::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub public_key: PublicKey,
}

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAddressList {
    pub sequencer_list: Vec<PublicKey>,
}

impl GetAddressList {
    pub const METHOD_NAME: &'static str = stringify!(GetAddressList);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub public_key: PublicKey,
    pub sequencer_rpc_address: RpcAddress,
}

impl Register {
    pub const METHOD_NAME: &'static str = stringify!(Register);
}
