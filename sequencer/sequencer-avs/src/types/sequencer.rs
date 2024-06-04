use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    BlockBuildingInProgress,
    OrderCommitment(OrderCommitment),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerAddress(String);

impl AsRef<[u8]> for SequencerAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for SequencerAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for SequencerAddress {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(PublicKey, Option<SequencerAddress>)>);

impl SequencerList {
    const ID: &'static str = stringify!(SequencerList);

    pub fn len(&self) -> u64 {
        self.0.len() as u64
    }

    pub fn iter(&self) -> core::slice::Iter<(PublicKey, Option<SequencerAddress>)> {
        self.0.iter()
    }

    pub fn get_by_index(&self, index: u64) -> Option<&(PublicKey, Option<SequencerAddress>)> {
        self.0.get(index as usize)
    }

    pub fn get(ssal_block_number: SsalBlockNumber) -> Result<Self, Error> {
        let key = (Self::ID, ssal_block_number);
        database().get(&key)
    }
}
