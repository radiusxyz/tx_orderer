use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(Address, Option<IpAddress>)>);

impl SequencerList {
    pub const ID: &'static str = stringify!(SequencerList);

    pub fn get(ssal_block_height: BlockHeight) -> Result<Self, DbError> {
        let key = (Self::ID, ssal_block_height);
        database()?.get(&key)
    }

    pub fn put(&self, ssal_block_height: BlockHeight) -> Result<(), DbError> {
        let key = (Self::ID, ssal_block_height);
        database()?.put(&key, self)
    }

    pub fn delete(ssal_block_height: BlockHeight) -> Result<(), DbError> {
        let key = (Self::ID, ssal_block_height);
        database()?.delete(&key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> Vec<(Address, Option<IpAddress>)> {
        self.0
    }
}
