use sequencer::types::{Address, IpAddress};

use crate::{models::SequencerModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRpcUrl {
    pub address: Address,
    pub rpc_url: IpAddress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRpcUrlResponse {
    pub success: bool,
}

impl RegisterRpcUrl {
    pub const METHOD_NAME: &'static str = "register_rpc_url";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<RegisterRpcUrlResponse, RpcError> {
        let parameter = parameter.parse::<RegisterRpcUrl>()?;

        // TODO: Remove this code
        // health_check(&parameter.rpc_url).await?;

        let sequencer = SequencerModel::new(parameter.address.into(), parameter.rpc_url.into());

        sequencer.put()?;

        Ok(RegisterRpcUrlResponse { success: true })
    }
}
