use super::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SequencingInfosModel;

impl SequencingInfosModel {
    const ID: &'static str = stringify!(SequencingInfosModel);

    pub fn initialize() -> Result<(), KvStoreError> {
        if Self::get().is_err() {
            let key = &Self::ID;
            let sequencing_infos = SequencingInfos::default();

            kvstore()?.put(key, &sequencing_infos)?
        }

        Ok(())
    }

    pub fn get() -> Result<SequencingInfos, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get(key)
    }

    pub fn get_or_default() -> Result<SequencingInfos, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut() -> Result<Lock<'static, SequencingInfos>, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get_mut(key)
    }

    pub fn put(sequencing_infos: &SequencingInfos) -> Result<(), KvStoreError> {
        let key = &Self::ID;

        kvstore()?.put(key, sequencing_infos)
    }
}
