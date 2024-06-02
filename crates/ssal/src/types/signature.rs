use primitives::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct SequencerSignature(ethers::types::Signature);

impl std::ops::Deref for SequencerSignature {
    type Target = ethers::types::Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SequencerSignature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ethers::types::Signature> for SequencerSignature {
    fn from(value: ethers::types::Signature) -> Self {
        Self(value)
    }
}
