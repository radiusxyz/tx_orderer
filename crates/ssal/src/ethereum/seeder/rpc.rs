use serde::{Deserialize, Serialize};

use crate::ethereum::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub public_key: H160,
}

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAddressList {
    pub sequencer_list: Vec<H160>,
}

impl GetAddressList {
    pub const METHOD_NAME: &'static str = stringify!(GetAddressList);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub public_key: H160,
    pub sequencer_rpc_address: String,
}

impl Register {
    pub const METHOD_NAME: &'static str = stringify!(Register);
}
