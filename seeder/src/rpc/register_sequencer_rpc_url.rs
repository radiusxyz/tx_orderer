use sequencer::types::{Address, IpAddress};

use super::prelude::*;
use crate::models::SequencerModel;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencerRpcUrl {
    pub sequencer_address: Address,
    pub sequencer_rpc_url: IpAddress,
}

impl RegisterSequencerRpcUrl {
    pub const METHOD_NAME: &'static str = "register_sequencer_rpc_url";
}

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<RegisterSequencerRpcUrl>()?;

    // TODO: Remove this code
    // health_check(&parameter.sequencer_rpc_url).await?;

    let sequencer = SequencerModel::new(
        parameter.sequencer_address.into(),
        parameter.sequencer_rpc_url.into(),
    );

    sequencer.put()?;

    Ok(())
}
