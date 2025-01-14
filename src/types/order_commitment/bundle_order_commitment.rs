use radius_sdk::signature::Signature;
use serde::{Deserialize, Serialize};

use super::SingleOrderCommitment;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BundleOrderCommitment {
    pub order_commitment_list: Vec<SingleOrderCommitment>,
    pub signature: Signature,
}
