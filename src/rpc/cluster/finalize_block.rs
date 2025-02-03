use std::str::FromStr;

use ethers_core::types::{Signature as EthSignature, H256};
use radius_sdk::{signature::ChainType, validation::symbiotic::types::Keccak256};
use tracing::{info, warn};

use crate::{
    rpc::prelude::{liveness::radius::initialize_new_cluster, *},
    task::build_block,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub finalize_block_message: FinalizeBlockMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub rollup_id: String,
    pub executor_address: Address,

    pub platform_block_height: u64,
    pub rollup_block_height: u64,

    pub block_creator_address: Address,
    pub next_block_creator_address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignMessage {
    pub rollup_id: String,
    pub executor_address: String,

    pub platform_block_height: u64,
    pub rollup_block_height: u64,

    pub block_creator_address: String,
    pub next_block_creator_address: String,
}

impl FinalizeBlock {
    pub fn get_executor_address(&self, chain_type: ChainType) -> Result<Address, RpcError> {
        // Serialize the sign message into JSON bytes
        let message_bytes = serde_json::to_vec(&SignMessage {
            rollup_id: self.finalize_block_message.rollup_id.clone(),
            executor_address: self.finalize_block_message.executor_address.as_hex_string(),
            platform_block_height: self.finalize_block_message.platform_block_height,
            rollup_block_height: self.finalize_block_message.rollup_block_height,
            block_creator_address: self
                .finalize_block_message
                .block_creator_address
                .as_hex_string(),
            next_block_creator_address: self
                .finalize_block_message
                .next_block_creator_address
                .as_hex_string(),
        })
        .map_err(|e| {
            Error::Internal(format!("Failed to serialize sign message: {:?}", e).into())
        })?;

        // Hash the serialized message using Keccak256
        let hash = {
            let mut hasher = Keccak256::new();
            hasher.update(message_bytes);
            let output = hasher.finalize();
            H256::from_slice(&output.as_slice())
        };

        // Recover the signer address from the signature and hash
        let recovered_address = EthSignature::from_str(&self.signature.as_hex_string())
            .map_err(|e| Error::Internal(format!("Invalid signature format: {:?}", e).into()))?
            .recover(hash)
            .map_err(|e| Error::Internal(format!("Failed to recover address: {:?}", e).into()))?;

        // Format the recovered address into a string and convert to Address type
        Ok(
            Address::from_str(chain_type, &format!("0x{:x}", recovered_address)).map_err(|e| {
                Error::Internal(format!("Invalid recovered address: {:?}", e).into())
            })?,
        )
    }
}

impl RpcParameter<AppState> for FinalizeBlock {
    type Response = ();

    fn method() -> &'static str {
        "finalize_block"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        info!("finalize block - executor address: {:?} / block creator (sequencer) address: {:?} / rollup_id: {:?} / platform block height: {:?} / rollup block height: {:?}",
        self.finalize_block_message.executor_address.as_hex_string(),
        self.finalize_block_message.block_creator_address.as_hex_string(),
        self.finalize_block_message.rollup_id,
        self.finalize_block_message.platform_block_height,
        self.finalize_block_message.rollup_block_height,);

        // Check the executor address
        let rollup = context
            .get_rollup(&self.finalize_block_message.rollup_id)
            .await?;

        let signer_address = self.get_executor_address(rollup.platform.into())?;

        rollup
            .executor_address_list
            .iter()
            .find(|&executor_address| signer_address == *executor_address)
            .ok_or_else(|| {
                warn!(
                    "Executor address not found: {:?}",
                    signer_address.as_hex_string()
                );
                Error::ExecutorAddressNotFound
            })?;

        let cluster = context
            .get_cluster(
                rollup.platform,
                rollup.service_provider,
                &rollup.cluster_id,
                self.finalize_block_message.platform_block_height,
            )
            .await;

        let cluster = if let Err(_) = cluster {
            let liveness_client = context
                .get_liveness_client::<liveness::radius::LivenessClient>(
                    rollup.platform,
                    rollup.service_provider,
                )
                .await?;

            let current_block_height = liveness_client
                .publisher()
                .get_block_number()
                .await
                .map_err(|_| Error::ClusterNotFound)?;

            let block_margin: u64 = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .map_err(|_| Error::ClusterNotFound)?
                .try_into()
                .map_err(|_| Error::ClusterNotFound)?;

            if self.finalize_block_message.platform_block_height + block_margin
                < current_block_height
            {
                return Err(Error::ClusterNotFound.into());
            }

            // Initialize new cluster if it doesn't exist
            initialize_new_cluster(
                context.clone(),
                liveness_client,
                &rollup.cluster_id,
                current_block_height,
                block_margin,
            )
            .await
            .map_err(Error::InitializeNewCluster)?;

            // Try to fetch the cluster again after initialization
            context
                .get_cluster(
                    rollup.platform,
                    rollup.service_provider,
                    &rollup.cluster_id,
                    self.finalize_block_message.platform_block_height,
                )
                .await?
        } else {
            cluster?
        };

        let transaction_count = self
            .finalize_block(context.clone(), &cluster, &rollup)
            .await?;

        build_block(
            context,
            cluster,
            self.finalize_block_message,
            self.signature,
            rollup.encrypted_transaction_type.clone(),
            transaction_count,
        );

        Ok(())
    }
}

impl FinalizeBlock {
    pub async fn finalize_block(
        &self,
        context: AppState,
        cluster: &Cluster,
        rollup: &Rollup,
    ) -> Result<u64, RpcError> {
        let next_rollup_block_height = self.finalize_block_message.rollup_block_height + 1;

        let signer = context.get_signer(rollup.platform).await?;
        let sequencer_address = signer.address().clone();
        let is_leader = sequencer_address == self.finalize_block_message.next_block_creator_address;

        let mut transaction_count = 0;

        if let Ok(mut locked_rollup_metadata) = context
            .get_mut_rollup_metadata(&self.finalize_block_message.rollup_id)
            .await
        {
            transaction_count = locked_rollup_metadata.transaction_order; // 2156

            locked_rollup_metadata.rollup_block_height = next_rollup_block_height;
            locked_rollup_metadata.platform_block_height =
                self.finalize_block_message.platform_block_height;
            locked_rollup_metadata.is_leader = is_leader;

            if let Some(sequencer_rpc_info) = cluster
                .get_sequencer_rpc_info(&self.finalize_block_message.next_block_creator_address)
            {
                locked_rollup_metadata.leader_sequencer_rpc_info = sequencer_rpc_info;
            } else {
                tracing::error!("Sequencer RPC info not found");
                return Err(Error::SignerNotFound)?;
            }

            locked_rollup_metadata.new_merkle_tree();
        } else {
            let mut rollup_metadata = RollupMetadata::default();

            rollup_metadata.cluster_id = rollup.cluster_id.to_string();

            rollup_metadata.rollup_block_height = next_rollup_block_height;
            rollup_metadata.platform_block_height =
                self.finalize_block_message.platform_block_height;
            rollup_metadata.is_leader = is_leader;

            if let Some(sequencer_rpc_info) = cluster
                .get_sequencer_rpc_info(&self.finalize_block_message.next_block_creator_address)
            {
                rollup_metadata.leader_sequencer_rpc_info = sequencer_rpc_info;
            } else {
                tracing::error!("Sequencer RPC info not found");
                return Err(Error::SignerNotFound)?;
            }

            rollup_metadata.new_merkle_tree();

            rollup_metadata.put(&self.finalize_block_message.rollup_id)?;
            context
                .add_rollup_metadata(&self.finalize_block_message.rollup_id, rollup_metadata)
                .await?;
        }

        Ok(transaction_count)
    }
}
