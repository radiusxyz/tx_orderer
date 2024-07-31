use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterModel {
    proposer_set_id: ProposerSetId,
    pub sequencer_addresses: Addresses,
}

impl ClusterModel {
    pub fn new(proposer_set_id: ProposerSetId) -> Self {
        Self {
            proposer_set_id,
            sequencer_addresses: Addresses::new(),
        }
    }
}

impl ClusterModel {
    pub const ID: &'static str = stringify!(ClusterModel);

    pub fn get(proposer_set_id: ProposerSetId) -> Result<Self, database::Error> {
        let key = (Self::ID, proposer_set_id);
        database()?.get(&key)
    }

    pub fn get_mut(proposer_set_id: ProposerSetId) -> Result<Lock<'static, Self>, database::Error> {
        let key = (Self::ID, proposer_set_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        let key = (Self::ID, self.proposer_set_id.clone());
        database()?.put(&key, self)
    }
}
