use super::prelude::*;

// #[derive(Clone, Debug, Serialize, Deserialize, Default)]
// pub struct SequencingInfosModel;

// impl SequencingInfosModel {
//     const ID: &'static str = stringify!(SequencingInfosModel);

//     pub fn put(sequencing_infos: &SequencingInfos) -> Result<(),
// KvStoreError> {         let key = &Self::ID;

//         kvstore()?.put(key, sequencing_infos)
//     }

//     pub fn get() -> Result<SequencingInfos, KvStoreError> {
//         let key = &Self::ID;

//         kvstore()?.get(key)
//     }

//     pub fn get_or_default() -> Result<SequencingInfos, KvStoreError> {
//         let key = &Self::ID;

//         kvstore()?.get_or_default(key)
//     }

//     pub fn get_mut_or_default() -> Result<Lock<'static, SequencingInfos>,
// KvStoreError> {         let key = &Self::ID;

//         kvstore()?.get_mut_or_default(key)
//     }
// }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SequencingInfoListModel;

impl SequencingInfoListModel {
    const ID: &'static str = stringify!(SequencingInfoListModel);

    pub fn get() -> Result<SequencingInfoList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get(key)
    }

    pub fn get_or_default() -> Result<SequencingInfoList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut_or_default() -> Result<Lock<'static, SequencingInfoList>, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_mut_or_default(key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencingInfoPayloadModel;

impl SequencingInfoPayloadModel {
    const ID: &'static str = stringify!(SequencingInfoPayloadModel);

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        value: &SequencingInfoPayload,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.put(key, value)
    }

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<SequencingInfoPayload, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get(key)
    }
}
