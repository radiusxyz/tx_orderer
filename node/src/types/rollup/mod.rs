mod rollup_metadata;
mod rollup_type;

use std::collections::{btree_set, BTreeSet};

pub use rollup_metadata::*;
pub use rollup_type::*;

use super::prelude::*;

pub type RollupId = String;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &RollupId))]
pub struct Rollup {
    pub cluster_id: ClusterId,
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,

    pub rollup_id: RollupId,
    pub rollup_type: RollupType,
    pub encrypted_transaction_type: EncryptedTransactionType,
    pub order_commitment_type: OrderCommitmentType,

    #[serde(serialize_with = "serialize_address")]
    pub owner: Address,

    pub validation_info: ValidationInfo,

    #[serde(serialize_with = "serialize_address_list")]
    pub executor_address_list: Vec<Address>,

    pub max_gas_limit: u64,
    pub max_transaction_count_per_batch: u64,
}

impl Rollup {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollup_id: RollupId,
        rollup_type: RollupType,
        encrypted_transaction_type: EncryptedTransactionType,

        owner: Address,
        validation_info: ValidationInfo,
        order_commitment_type: OrderCommitmentType,
        executor_address_list: Vec<Address>,

        cluster_id: ClusterId,

        platform: Platform,
        liveness_service_provider: LivenessServiceProvider,
    ) -> Self {
        Self {
            rollup_id,
            rollup_type,
            encrypted_transaction_type,
            owner,
            validation_info,
            order_commitment_type,
            executor_address_list,
            cluster_id,
            platform,
            liveness_service_provider,

            max_gas_limit: 0,                   // TODO
            max_transaction_count_per_batch: 4, // TODO
        }
    }

    pub fn set_executor_address_list(&mut self, executor_address_list: Vec<Address>) {
        self.executor_address_list = executor_address_list;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key())]
pub struct RollupIdList(BTreeSet<RollupId>);

impl RollupIdList {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn set(&mut self, rollup_id_list: Vec<RollupId>) {
        self.0 = rollup_id_list.into_iter().collect();
    }

    pub fn insert(&mut self, cluster_id: impl AsRef<str>) {
        self.0.insert(cluster_id.as_ref().into());
    }

    pub fn remove(&mut self, cluster_id: impl AsRef<str>) {
        self.0.remove(cluster_id.as_ref());
    }

    pub fn iter(&self) -> btree_set::Iter<'_, RollupId> {
        self.0.iter()
    }
}
