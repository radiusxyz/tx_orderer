use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupIdListModel {
    rollup_id_list: RollupIdList,
}

impl RollupIdListModel {
    pub fn new(rollup_id_list: RollupIdList) -> Self {
        Self { rollup_id_list }
    }

    pub fn rollup_id_list(self) -> RollupIdList {
        self.rollup_id_list
    }

    pub fn is_exist_rollup_id(&self, rollup_id: &String) -> bool {
        self.rollup_id_list.contains(rollup_id)
    }

    pub fn add_rollup_id(&mut self, rollup_id: String) {
        let is_exist_rollup_id = self.rollup_id_list.contains(&rollup_id);

        if !is_exist_rollup_id {
            self.rollup_id_list.push(rollup_id);
        }
    }

    pub fn update_rollup_id_list(&mut self, rollup_id_list: RollupIdList) {
        self.rollup_id_list = rollup_id_list;
    }
}

impl RollupIdListModel {
    pub const ID: &'static str = stringify!(RollupIdListModel);

    pub fn get() -> Result<Self, KvStoreError> {
        let key = Self::ID;
        kvstore()?.get(&key)
    }

    // change func name or separate
    pub fn get_mut() -> Result<Lock<'static, Self>, KvStoreError> {
        let key = Self::ID;
        kvstore()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), KvStoreError> {
        let key = Self::ID;
        kvstore()?.put(&key, self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupMetadataModel {
    rollup_id: String,
    rollup_metadata: RollupMetadata,
}

impl RollupMetadataModel {
    pub fn new(rollup_id: String, rollup_metadata: RollupMetadata) -> Self {
        Self {
            rollup_id,
            rollup_metadata,
        }
    }

    pub fn rollup_id(&self) -> &String {
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

    pub fn get(rollup_id: &String) -> Result<Self, KvStoreError> {
        let key = (Self::ID, rollup_id);
        kvstore()?.get(&key)
    }

    pub fn get_mut(rollup_id: &String) -> Result<Lock<'static, Self>, KvStoreError> {
        let key = (Self::ID, rollup_id);
        kvstore()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), KvStoreError> {
        let key = (Self::ID, self.rollup_id());
        kvstore()?.put(&key, self)
    }
}
