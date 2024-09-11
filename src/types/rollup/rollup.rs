use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rollup {
    rollup_id: String,
    rollup_type: RollupType,

    owner: String,
    validation_info: ValidationInfo,
    order_commitment_type: OrderCommitmentType,
    executor_address_list: Vec<String>,
}

impl Rollup {
    pub fn new(
        rollup_id: String,
        rollup_type: RollupType,
        owner: String,
        validation_info: ValidationInfo,
        order_commitment_type: OrderCommitmentType,
        executor_address_list: Vec<String>,
    ) -> Self {
        Self {
            rollup_id,
            rollup_type,
            owner,
            validation_info,
            order_commitment_type,
            executor_address_list,
        }
    }

    pub fn rollup_id(&self) -> &String {
        &self.rollup_id
    }
}
