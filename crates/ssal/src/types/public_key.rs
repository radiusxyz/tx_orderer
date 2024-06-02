use primitives::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct SequencerPublicKey(pub ethers::types::H160);

impl std::ops::Deref for SequencerPublicKey {
    type Target = ethers::types::H160;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SequencerPublicKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ethers::types::H160> for SequencerPublicKey {
    fn from(value: ethers::types::H160) -> Self {
        Self(value)
    }
}
