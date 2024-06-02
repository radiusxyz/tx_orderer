use std::net::SocketAddr;

use primitives::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct SequencerAddress(String);

impl From<SocketAddr> for SequencerAddress {
    fn from(value: SocketAddr) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for SequencerAddress {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for SequencerAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
