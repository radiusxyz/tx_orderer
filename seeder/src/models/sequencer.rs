use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerModel {
    address: Address,
    pub rpc_url: IpAddress,
}

impl SequencerModel {
    pub fn new(address: Address, rpc_url: IpAddress) -> Self {
        Self { address, rpc_url }
    }
}
impl SequencerModel {
    pub const ID: &'static str = stringify!(SequencerModel);

    pub fn get(address: Address) -> Result<Self, database::Error> {
        let key = (Self::ID, address);
        database()?.get(&key)
    }

    pub fn get_mut(address: Address) -> Result<Lock<'static, Self>, database::Error> {
        let key = (Self::ID, address);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        let key = (Self::ID, self.address.clone());
        database()?.put(&key, self)
    }
}
