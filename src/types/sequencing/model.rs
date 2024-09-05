use super::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SequencingInfoListModel;

impl SequencingInfoListModel {
    const ID: &'static str = stringify!(SequencingInfoListModel);

    pub fn get() -> Result<SequencingInfoList, KvStoreError> {
        kvstore()?.get(&Self::ID)
    }

    pub fn get_mut() -> Result<Lock<'static, SequencingInfoList>, KvStoreError> {
        kvstore()?.get_mut(&Self::ID)
    }

    pub fn put(sequencing_info_list: &SequencingInfoList) -> Result<(), KvStoreError> {
        kvstore()?.put(&Self::ID, sequencing_info_list)
    }
}
