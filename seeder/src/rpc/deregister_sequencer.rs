use sequencer::types::{Address, ProposerSetId};

use super::prelude::*;
use crate::task::event_listener::deregister_sequencer;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterSequencer {
    pub proposer_set_id: ProposerSetId,
    pub sequencer_address: Address,
}

impl DeregisterSequencer {
    pub const METHOD_NAME: &'static str = "deregister_sequencer";
}

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<DeregisterSequencer>()?;

    deregister_sequencer(parameter.proposer_set_id, parameter.sequencer_address)?;

    Ok(())
}
