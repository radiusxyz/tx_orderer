use sequencer::types::{AddressList, IpAddress};
use tracing::info;

use super::prelude::*;
use crate::models::{ClusterModel, SequencerModel};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlList {
    pub proposer_set_id: String,
}

impl GetSequencerRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url_list";
}

pub async fn handler(
    parameter: RpcParameter,
    _context: Arc<()>,
) -> Result<Vec<Option<IpAddress>>, RpcError> {
    let parameter = parameter.parse::<GetSequencerRpcUrlList>()?;

    info!(
        "get_sequencer_rpc_url_list: {:?}",
        parameter.proposer_set_id
    );

    let cluster_model = ClusterModel::get(parameter.proposer_set_id.clone())?;

    let sequencer_address_list = cluster_model
        .sequencer_addresses
        .keys()
        .cloned()
        .collect::<AddressList>();

    println!("sequencer_address_list: {:?}", sequencer_address_list);

    let sequencer_list = sequencer_address_list
        .iter()
        .map(|sequencer_address| {
            SequencerModel::get(sequencer_address.clone())
                .map(|sequencer_model| Some(sequencer_model.rpc_url))
                .map_err(|e| e.into())
        })
        .collect::<Result<Vec<Option<IpAddress>>, RpcError>>()?;

    Ok(sequencer_list)
}
