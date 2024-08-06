use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupModel {
    rollup: Rollup,

    sequencing_info_key: SequencingInfoKey,
    cluster_id: ClusterId,
}

impl RollupModel {
    pub fn rollup(&self) -> &Rollup {
        &self.rollup
    }

    pub fn sequencing_info_key(&self) -> &SequencingInfoKey {
        &self.sequencing_info_key
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.cluster_id
    }
}

impl RollupModel {
    pub const ID: &'static str = stringify!(RollupModel);

    pub fn new(
        rollup: Rollup,
        sequencing_info_key: SequencingInfoKey,
        cluster_id: ClusterId,
    ) -> Self {
        Self {
            rollup,
            sequencing_info_key,
            cluster_id,
        }
    }

    pub fn get(rollup_id: &RollupId) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.rollup.rollup_id());
        database()?.put(&key, self)
    }
}
