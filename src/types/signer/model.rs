use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignerListModel;

impl SignerListModel {
    const ID: &'static str = stringify!(SignerListModel);

    pub fn get() -> Result<SignerList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get(key)
    }

    pub fn get_or_default() -> Result<SignerList, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut_or_default() -> Result<Lock<'static, SignerList>, KvStoreError> {
        let key = &(Self::ID);

        kvstore()?.get_mut_or_default(key)
    }
}
