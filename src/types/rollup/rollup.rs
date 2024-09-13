use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rollup {
    rollup_id: String,
    rollup_type: RollupType,
    encrypted_transaction_type: EncryptedTransactionType,

    owner: String,
    validation_info: ValidationInfo,
    order_commitment_type: OrderCommitmentType,
    executor_address_list: Vec<String>,

    cluster_id: String,
}

impl Rollup {
    pub fn new(
        rollup_id: String,
        rollup_type: RollupType,
        encrypted_transaction_type: EncryptedTransactionType,

        owner: String,
        validation_info: ValidationInfo,
        order_commitment_type: OrderCommitmentType,
        executor_address_list: Vec<String>,

        cluster_id: String,
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
        }
    }

    pub fn rollup_id(&self) -> &String {
        &self.rollup_id
    }

    pub fn rollup_type(&self) -> RollupType {
        self.rollup_type
    }

    pub fn encrypted_transaction_type(&self) -> EncryptedTransactionType {
        self.encrypted_transaction_type
    }

    pub fn order_commitment_type(&self) -> OrderCommitmentType {
        self.order_commitment_type
    }

    pub fn cluster_id(&self) -> &String {
        &self.cluster_id
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptedTransactionType {
    Pvde,
    Skde,
    NotSupport,
}
