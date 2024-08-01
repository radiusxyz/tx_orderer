use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SsalBlockModel {
    pub block_height: BlockHeight,
}

impl SsalBlockModel {
    pub const ID: &'static str = stringify!(SsalBlockModel);

    pub fn get() -> Result<Self, DbError> {
        database()?.get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), DbError> {
        database()?.put(&Self::ID, self)
    }
}
