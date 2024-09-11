use super::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SequencingInfoListModel;

impl SequencingInfoListModel {
    const ID: &'static str = stringify!(SequencingInfoListModel);

    pub fn get() -> Result<SequencingInfoList, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get(key)
    }

    pub fn get_or_default() -> Result<SequencingInfoList, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut() -> Result<Lock<'static, SequencingInfoList>, KvStoreError> {
        let key = &Self::ID;

        kvstore()?.get_mut(key)
    }

    pub fn put(sequencing_info_list: &SequencingInfoList) -> Result<(), KvStoreError> {
        let key = &Self::ID;

        kvstore()?.put(key, sequencing_info_list)
    }
}
