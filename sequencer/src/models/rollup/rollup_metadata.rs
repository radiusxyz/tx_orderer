use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupMetadataModel {
    rollup_id: RollupId,
    rollup_metadata: RollupMetadata,
}

impl RollupMetadataModel {
    pub fn new(rollup_id: RollupId, rollup_metadata: RollupMetadata) -> Self {
        Self {
            rollup_id,
            rollup_metadata,
        }
    }

    pub fn rollup_id(&self) -> &RollupId {
        &self.rollup_id
    }

    pub fn rollup_metadata(&self) -> &RollupMetadata {
        &self.rollup_metadata
    }

    pub fn update_rollup_metadata(&mut self, rollup_metadata: RollupMetadata) {
        self.rollup_metadata = rollup_metadata;
    }

    pub fn issue_new_block(&mut self) {
        self.rollup_metadata.issue_new_block()
    }
}

impl RollupMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(rollup_id: &RollupId) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.rollup_id());
        database()?.put(&key, self)
    }
}
