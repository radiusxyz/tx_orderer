use radius_sdk::{kvstore::KvStore, signature::Address};
use serde::{Deserialize, Serialize};

use crate::{client::liveness_service_manager::seeder::SequencerRpcInfo, error::Error, types::*};

const PREVIOUS_DATABASE_VERSION: &'static str = "v0.0.1";
const CURRENT_DATABASE_VERSION: &'static str = "v0.0.2";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OldRollupMetadata {
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub cluster_id: String,
    pub platform_block_height: u64,
    pub is_leader: bool,
    pub leader_sequencer_rpc_info: SequencerRpcInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

pub async fn migrate_rollup_data(kv_store: KvStore) -> Result<(), Error> {
    let mut version: Version = kv_store.get(&("Version",)).unwrap_or_default();

    if version.database_version != PREVIOUS_DATABASE_VERSION {
        tracing::error!(
            "Database version mismatch. Expected: {:?}, Found: {:?}",
            PREVIOUS_DATABASE_VERSION,
            version.database_version
        );
        return Err(Error::DatabaseVersionMismatch);
    }

    tracing::info!(
        "Migrating database from version {:?} to {:?}",
        PREVIOUS_DATABASE_VERSION,
        CURRENT_DATABASE_VERSION
    );

    let rollup_id_list: RollupIdList = kv_store.get(&("RollupIdList",)).map_err(Error::Database)?;

    for rollup_id in rollup_id_list.iter() {
        tracing::info!("Checking data - rollup_id: {:?}", rollup_id);
        migrate_rollup(&kv_store, rollup_id)?;
        migrate_rollup_metadata(&kv_store, rollup_id)?;
    }

    version.database_version = CURRENT_DATABASE_VERSION.to_owned();
    kv_store
        .put(&("Version",), &version)
        .map_err(Error::Database)?;
    tracing::info!("Database version updated to {:?}", CURRENT_DATABASE_VERSION);

    Ok(())
}

fn migrate_rollup(kv_store: &KvStore, rollup_id: &str) -> Result<(), Error> {
    if kv_store
        .get::<(&str, &str), Rollup>(&("Rollup", rollup_id))
        .is_err()
    {
        tracing::info!("Migrating old Rollup data: {:?}", rollup_id);

        let old_rollup: OldRollup = kv_store
            .get(&("Rollup", rollup_id))
            .map_err(Error::Database)?;
        let new_rollup = Rollup {
            cluster_id: old_rollup.cluster_id,
            platform: old_rollup.platform,
            service_provider: old_rollup.service_provider,
            rollup_id: old_rollup.rollup_id,
            rollup_type: old_rollup.rollup_type,
            encrypted_transaction_type: old_rollup.encrypted_transaction_type,
            order_commitment_type: old_rollup.order_commitment_type,
            owner: old_rollup.owner,
            validation_info: old_rollup.validation_info,
            executor_address_list: old_rollup.executor_address_list,
            max_gas_limit: 0,
        };

        kv_store
            .put(&("Rollup", rollup_id), &new_rollup)
            .map_err(Error::Database)?;
        tracing::info!("Migration of Rollup {:?} completed", rollup_id);
    }
    Ok(())
}

fn migrate_rollup_metadata(kv_store: &KvStore, rollup_id: &str) -> Result<(), Error> {
    if kv_store
        .get::<(&str, &str), RollupMetadata>(&("RollupMetadata", rollup_id))
        .is_err()
    {
        tracing::info!("Migrating old RollupMetadata data: {:?}", rollup_id);

        let old_metadata: OldRollupMetadata = kv_store
            .get(&("RollupMetadata", rollup_id))
            .map_err(Error::Database)?;

        let new_metadata = RollupMetadata {
            rollup_block_height: old_metadata.rollup_block_height,
            transaction_order: old_metadata.transaction_order,
            cluster_id: old_metadata.cluster_id,
            platform_block_height: old_metadata.platform_block_height,
            is_leader: old_metadata.is_leader,
            leader_sequencer_rpc_info: old_metadata.leader_sequencer_rpc_info,
            max_gas_limit: 0,
            current_gas: 0,
        };

        kv_store
            .put(&("RollupMetadata", rollup_id), &new_metadata)
            .map_err(Error::Database)?;

        tracing::info!("Migration of RollupMetadata {:?} completed", rollup_id);
    }
    Ok(())
}
