use std::collections::HashMap;

use sequencer::types::{Address, AddressList, IpAddress, ProposerSetId};
use tracing::info;

use super::prelude::*;
use crate::models::{ClusterModel, SequencerModel};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrls {
    pub proposer_set_id: ProposerSetId,
}

impl GetSequencerRpcUrls {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_urls";
}

pub async fn handler(
    parameter: RpcParameter,
    _context: Arc<()>,
) -> Result<HashMap<Address, IpAddress>, RpcError> {
    let parameter = parameter.parse::<GetSequencerRpcUrls>()?;

    info!("get_sequencer_rpc_urls: {:?}", parameter.proposer_set_id);

    let cluster_model = ClusterModel::get(parameter.proposer_set_id.clone())?;

    let sequencer_address_list = cluster_model
        .sequencer_addresses
        .keys()
        .cloned()
        .collect::<AddressList>();

    println!("sequencer_address_list: {:?}", sequencer_address_list);

    let sequencer_rpc_urls = sequencer_address_list
        .iter()
        .filter_map(
            |sequencer_address| match SequencerModel::get(sequencer_address.clone()) {
                Ok(sequencer_model) => {
                    if let Some(rpc_url) = sequencer_model.rpc_url {
                        Some((sequencer_model.address, rpc_url))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
        )
        .collect::<HashMap<Address, IpAddress>>();

    Ok(sequencer_rpc_urls)
}
