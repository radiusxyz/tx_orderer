use serde::{Deserialize, Serialize};

use crate::avs::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub sequencer_address: Address,
    pub sequencer_rpc_url: String,
}

impl Register {
    pub const METHOD_NAME: &'static str = stringify!(Register);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    pub sequencer_address: Address,
}

impl Deregister {
    pub const METHOD_NAME: &'static str = stringify!(Deregister);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlList {
    pub sequencer_address_list: Vec<Address>,
}

impl GetSequencerRpcUrlList {
    pub const METHOD_NAME: &'static str = stringify!(GetAddressList);
}
