use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionHashOrderCommitment(String);

impl TransactionHashOrderCommitment {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}
