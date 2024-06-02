use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey(ethers::types::H160);

impl std::ops::Deref for PublicKey {
    type Target = ethers::types::H160;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PublicKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ethers::types::H160> for PublicKey {
    fn from(value: ethers::types::H160) -> Self {
        Self(value)
    }
}
