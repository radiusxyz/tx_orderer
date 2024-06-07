use std::str::FromStr;

use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

impl TryFrom<&str> for PublicKey {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let public_key = ethers::types::H160::from_str(value)
            .map_err(|error| Error::boxed(ErrorKind::ParsePublicKey, error))?;
        Ok(Self(public_key))
    }
}
