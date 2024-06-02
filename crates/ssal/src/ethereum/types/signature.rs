use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature(ethers::types::Signature);

impl std::ops::Deref for Signature {
    type Target = ethers::types::Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ethers::types::Signature> for Signature {
    fn from(value: ethers::types::Signature) -> Self {
        Self(value)
    }
}
