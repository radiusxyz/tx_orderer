use primitives::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct ClusterId([u8; 32]);

impl From<[u8; 32]> for ClusterId {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}
