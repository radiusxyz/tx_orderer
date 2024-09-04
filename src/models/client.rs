use super::prelude::*;

/// 09/05
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientModel;

impl ClientModel {
    const ID: &'static str = stringify!(ClientModel);

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<SequencingInfoPayload, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get(key)
    }
}
