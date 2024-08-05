use sequencer::types::{Address, IpAddress};
use tracing::info;

use crate::{models::SequencerModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrl {
    pub address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrlResponse {
    pub rpc_url: Option<IpAddress>,
}

impl GetRpcUrl {
    pub const METHOD_NAME: &'static str = "get_rpc_url";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetRpcUrlResponse, RpcError> {
        let parameter = parameter.parse::<GetRpcUrl>()?;

        info!("get_rpc_url: {:?}", parameter.address);

        let sequencer_model = SequencerModel::get(parameter.address)?;

        Ok(GetRpcUrlResponse {
            rpc_url: sequencer_model.rpc_url,
        })
    }
}
