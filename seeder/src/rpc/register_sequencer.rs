use sequencer::types::{Address, ProposerSetId};

use super::prelude::*;
use crate::task::event_listener::register_sequencer;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencer {
    pub proposer_set_id: ProposerSetId,
    pub sequencer_address: Address,
}

impl RegisterSequencer {
    pub const METHOD_NAME: &'static str = "register_sequencer";
}

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<RegisterSequencer>()?;

    register_sequencer(parameter.proposer_set_id, parameter.sequencer_address)?;

    Ok(())
}
