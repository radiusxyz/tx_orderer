use serde::{Deserialize, Serialize};

use super::{SignOrderCommitment, TransactionHashOrderCommitment};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum SingleOrderCommitment {
    TransactionHash(TransactionHashOrderCommitment),
    Sign(SignOrderCommitment),
}

impl Default for SingleOrderCommitment {
    fn default() -> Self {
        Self::TransactionHash(TransactionHashOrderCommitment::default())
    }
}
