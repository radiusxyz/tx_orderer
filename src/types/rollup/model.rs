use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupMetadataModel;

impl RollupMetadataModel {
    const ID: &'static str = stringify!(RollupMetadata);

    pub fn get_mut(rollup_id: &String) -> Result<Lock<RollupMetadata>, KvStoreError> {
        let key = &(Self::ID, rollup_id);

        kvstore()?.get_mut(key)
    }

    pub fn get_or_default(rollup_id: &String) -> Result<RollupMetadata, KvStoreError> {
        let key = &(Self::ID, rollup_id);

        kvstore()?.get_or_default(key)
    }

    pub fn put(rollup_id: &String, rollup_metadata: &RollupMetadata) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id);

        kvstore()?.put(key, rollup_metadata)
    }
}
