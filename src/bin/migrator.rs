use radius_sdk::{
    kvstore::{KvStoreBuilder, KvStoreError, Model},
    signature::Address,
};
use sequencer::{client::liveness::seeder::SequencerRpcInfo, types::*};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), KvStoreError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let database_path = args.get(0).expect("Provide the database path.").to_owned();
    KvStoreBuilder::default().build(database_path)?.init();

    let rollup_id = "radius_rollup";
    if let Some(rollup) = OldRollup::get(rollup_id).ok() {
        let new_rollup = Rollup {
            cluster_id: rollup.cluster_id,
            platform: rollup.platform,
            service_provider: rollup.service_provider,
            rollup_id: rollup.rollup_id,
            rollup_type: rollup.rollup_type,
            encrypted_transaction_type: rollup.encrypted_transaction_type,
            order_commitment_type: rollup.order_commitment_type,
            owner: rollup.owner,
            validation_info: rollup.validation_info,
            executor_address_list: rollup.executor_address_list,
            max_gas_limit: 0,
        };
        new_rollup.put(rollup_id)?;
    }

    if let Some(rollup_metadata) = OldRollupMetadata::get(rollup_id).ok() {
        let new_rollup_metadata = RollupMetadata {
            rollup_block_height: rollup_metadata.rollup_block_height,
            transaction_order: rollup_metadata.transaction_order,
            cluster_id: rollup_metadata.cluster_id,
            platform_block_height: rollup_metadata.platform_block_height,
            is_leader: rollup_metadata.is_leader,
            leader_sequencer_rpc_info: rollup_metadata.leader_sequencer_rpc_info,
            max_gas_limit: 0,
            current_gas: 0,
        };
        new_rollup_metadata.put(rollup_id)?;
    }

    Ok(())
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct OldRollupMetadata {
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub cluster_id: String,
    pub platform_block_height: u64,
    pub is_leader: bool,
    pub leader_sequencer_rpc_info: SequencerRpcInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct OldRollup {
    pub cluster_id: String,
    pub platform: Platform,
    pub service_provider: ServiceProvider,

    pub rollup_id: String,
    pub rollup_type: RollupType,
    pub encrypted_transaction_type: EncryptedTransactionType,
    pub order_commitment_type: OrderCommitmentType,

    #[serde(serialize_with = "serialize_address")]
    pub owner: Address,

    pub validation_info: RollupValidationInfo,

    #[serde(serialize_with = "serialize_address_list")]
    pub executor_address_list: Vec<Address>,
}
