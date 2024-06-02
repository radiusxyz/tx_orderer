use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

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
