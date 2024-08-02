use sequencer::types::{ClusterType, ProposerSetId};

use super::prelude::*;
use crate::{models::ClusterModel, task::event_listener::initialize_cluster};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitializeProposerSet {
    pub proposer_set_id: ProposerSetId,
}

impl InitializeProposerSet {
    pub const METHOD_NAME: &'static str = "initialize_proposer_set";
}

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<InitializeProposerSet>()?;

    match ClusterModel::get(&parameter.proposer_set_id) {
        Ok(_) => {} // TODO: Return an error
        Err(_) => {
            initialize_cluster(parameter.proposer_set_id, ClusterType::Local)?;
        }
    }

    Ok(())
}
