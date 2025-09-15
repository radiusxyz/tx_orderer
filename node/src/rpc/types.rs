use tx_orderer_primitives::{Address, ChainType, serialize_address};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxOrdererRpcInfo {
    #[serde(serialize_with = "serialize_address")]
    pub tx_orderer_address: Address,
    pub external_rpc_url: Option<String>,
    pub cluster_rpc_url: Option<String>,
}

impl Default for TxOrdererRpcInfo {
    fn default() -> Self {
        Self {
            tx_orderer_address: Address::from_slice(ChainType::Ethereum, &[0u8; 20]).unwrap(),
            external_rpc_url: None,
            cluster_rpc_url: None,
        }
    }
}