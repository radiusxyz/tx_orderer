use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterModel {
    proposer_set_id: ProposerSetId,
    pub sequencer_addresses: Addresses,
    pub cluster_type: ClusterType,
}

impl ClusterModel {
    pub fn new(proposer_set_id: ProposerSetId, cluster_type: ClusterType) -> Self {
        Self {
            proposer_set_id,
            sequencer_addresses: Addresses::new(),
            cluster_type,
        }
    }
}

impl ClusterModel {
    pub const ID: &'static str = stringify!(ClusterModel);

    pub fn get(proposer_set_id: ProposerSetId) -> Result<Self, DbError> {
        let key = (Self::ID, proposer_set_id);
        database()?.get(&key)
    }

    pub fn get_mut(proposer_set_id: ProposerSetId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, proposer_set_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.proposer_set_id.clone());
        database()?.put(&key, self)
    }
}
