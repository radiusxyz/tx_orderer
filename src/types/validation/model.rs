use super::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidationInfoListModel;

impl ValidationInfoListModel {
    const ID: &'static str = stringify!(ValidationInfoListModel);

    pub fn get() -> Result<ValidationInfoList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get(key)
    }

    pub fn get_or_default() -> Result<ValidationInfoList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut_or_default() -> Result<Lock<'static, ValidationInfoList>, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_mut_or_default(key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationInfoPayloadModel;

impl ValidationInfoPayloadModel {
    const ID: &'static str = stringify!(ValidationInfoPayloadModel);

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        value: &ValidationInfoPayload,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.put(key, value)
    }

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<ValidationInfoPayload, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get(key)
    }
}
