use crate::models::prelude::*;

/// 09/05
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SequencingInfoListModel;

impl SequencingInfoListModel {
    const ID: &'static str = stringify!(SequencingInfoListModel);

    pub fn get() -> Result<SequencingInfoList, KvStoreError> {
        kvstore()?.get(&Self::ID)
    }

    pub fn put(sequencing_info_list: &SequencingInfoList) -> Result<(), KvStoreError> {
        kvstore()?.put(&Self::ID, sequencing_info_list)
    }
}
