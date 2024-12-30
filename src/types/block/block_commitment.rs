use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockCommitment(String);

impl Default for BlockCommitment {
    fn default() -> Self {
        Self(const_hex::encode_prefixed([0; 32]))
    }
}

impl From<[u8; 32]> for BlockCommitment {
    fn from(value: [u8; 32]) -> Self {
        Self(const_hex::encode_prefixed(value))
    }
}

impl From<&str> for BlockCommitment {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for BlockCommitment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl BlockCommitment {
    pub fn as_bytes(self) -> Result<Vec<u8>, const_hex::FromHexError> {
        const_hex::decode(self.0)
    }

    pub fn as_hex_string(self) -> String {
        self.0
    }
}
