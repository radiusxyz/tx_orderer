use pvde::encryption::poseidon_encryption::encrypt;

use crate::rpc::{cluster::SyncBlock, prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    // platform: Platform,
    // service_provider: ServiceProvider,
    // cluster_id: String,
    // chain_type: ChainType,
    address: Vec<u8>,
    rollup_id: String,
    liveness_block_height: u64,
    rollup_block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub message: FinalizeBlockMessage,
    pub signature: Signature,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup = RollupModel::get(&parameter.message.rollup_id)?;

        // // verify siganture
        // parameter.signature.verify_message(
        //     rollup.rollup_type().into(),
        //     &parameter.message,
        //     parameter.message.address.clone(),
        // )?;

        // let cluster_info = ClusterInfoModel::get(
        //     parameter.message.liveness_block_height,
        //     &parameter.message.rollup_id,
        // )?;

        match RollupMetadataModel::get_mut(&parameter.message.rollup_id) {
            Ok(mut rollup_metadata) => {
                let transaction_counts = rollup_metadata.transaction_order();

                if matches!(
                    rollup.encrypted_transaction_type(),
                    EncryptedTransactionType::Skde
                ) {
                    if matches!(*rollup.order_commitment_type(), OrderCommitmentType::TxHash) {
                        for transaction_order in 0..transaction_counts {
                            let encrypted_transaction =
                                EncryptedTransactionModel::get_with_order_commitment(
                                    &parameter.message.rollup_id,
                                    parameter.message.rollup_block_height,
                                    transaction_order,
                                )?;

                            match encrypted_transaction {
                                EncryptedTransaction::Skde(transaction) => {
                                    match RawTransactionModel::get(
                                        &parameter.message.rollup_id,
                                        parameter.message.rollup_block_height,
                                        transaction_order,
                                    ) {
                                        Ok(_raw_transaction) => continue,
                                        Err(error) => {
                                            if error.is_none_type() {
                                                let decryption_key = _context
                                                    .key_management_system_client()
                                                    .get_decryption_key(transaction.key_id())
                                                    .await?
                                                    .key;
                                                // decrypt -> raw_transaction
                                                // let raw_transaction =
                                                // RawTransaction::new(
                                                //     open_data,
                                                //     plaindata //
                                                // encrypted_transaction.
                                                // decrypt(&decryption_key,
                                                // rollup.encrypted_transaction_type()),
                                                // );

                                                // // update db to raw
                                                // transaction
                                                // RawTransactionModel::put(
                                                //     &parameter.message.
                                                // rollup_id,
                                                //     parameter.message.
                                                // rollup_block_height,
                                                //     transaction_order,
                                                //     &raw_transaction,
                                                // )?;

                                                // let block =
                                                // BlockModel::get_mut(
                                                //     &parameter.message.
                                                // rollup_id,
                                                //     parameter.message.
                                                // rollup_block_height,
                                                // )?;

                                                // block.
                                            } else {
                                                return Err(error.into());
                                            }
                                        }
                                    }
                                }
                                EncryptedTransaction::Pvde(_transaction) => continue,
                            }
                        }
                    }
                }

                let current_rollup_metadata =
                    rollup_metadata.issue_rollup_metadata(parameter.message.rollup_block_height);
                rollup_metadata.update()?;

                let cluster_info = ClusterInfoModel::get(
                    parameter.message.liveness_block_height,
                    &parameter.message.rollup_id,
                )?;

                if cluster_info.sequencer_list().is_empty() {
                    return Err(Error::EmptySequencerList.into());
                }

                let cluster_metadata = ClusterMetadata::new(
                    cluster_info.sequencer_list().len()
                        % parameter.message.rollup_block_height as usize,
                    cluster_info.my_index(),
                    cluster_info.sequencer_list().clone(),
                );
                ClusterMetadataModel::put(
                    &parameter.message.rollup_id,
                    parameter.message.rollup_block_height,
                    &cluster_metadata,
                )?;

                // Sync.
                Self::sync_block(
                    &parameter,
                    current_rollup_metadata.transaction_order(),
                    cluster_metadata,
                );
            }
            Err(error) => {
                if error.is_none_type() {
                    let mut rollup_metadata = RollupMetadata::default();
                    rollup_metadata.set_block_height(parameter.message.rollup_block_height);
                    RollupMetadataModel::put(&parameter.message.rollup_id, &rollup_metadata)?;

                    let cluster_info = ClusterInfoModel::get(
                        parameter.message.liveness_block_height,
                        &parameter.message.rollup_id,
                    )?;

                    if cluster_info.sequencer_list().is_empty() {
                        return Err(Error::EmptySequencerList.into());
                    }

                    let cluster_metadata = ClusterMetadata::new(
                        cluster_info.sequencer_list().len()
                            % parameter.message.rollup_block_height as usize,
                        cluster_info.my_index(),
                        cluster_info.sequencer_list().clone(),
                    );
                    ClusterMetadataModel::put(
                        &parameter.message.rollup_id,
                        parameter.message.rollup_block_height,
                        &cluster_metadata,
                    )?;

                    // Sync.
                    Self::sync_block(
                        &parameter,
                        rollup_metadata.transaction_order(),
                        cluster_metadata,
                    );
                } else {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    pub fn sync_block(parameter: &Self, transaction_order: u64, cluster_metadata: ClusterMetadata) {
        let parameter = parameter.clone();

        tokio::spawn(async move {
            let rpc_parameter = SyncBlock {
                rollup_id: parameter.message.rollup_id.clone(),
                liveness_block_height: parameter.message.liveness_block_height,
                rollup_block_height: parameter.message.rollup_block_height,
                transaction_order,
            };

            for sequencer_rpc_url in cluster_metadata.others() {
                let rpc_parameter = rpc_parameter.clone();

                if let Some(sequencer_rpc_url) = sequencer_rpc_url {
                    tokio::spawn(async move {
                        let client = RpcClient::new(sequencer_rpc_url).unwrap();
                        let _ = client
                            .request::<SyncBlock, ()>(SyncBlock::METHOD_NAME, rpc_parameter.clone())
                            .await;
                    });
                }
            }
        });
    }
}
