use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub rollup_id: String,
    pub liveness_block_height: u64, // TODO
    pub rollup_block_height: u64,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match RollupMetadataModel::get_mut(&parameter.rollup_id) {
            Ok(mut rollup_metadata_model) => {
                let current_rollup_metadata_model = rollup_metadata_model.clone();
                rollup_metadata_model.issue_new_block();
                rollup_metadata_model.update()?;
                Self::sync_block(current_rollup_metadata_model);

                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.is_none_type() {
                    let rollup_metadata = RollupMetadata::new(
                        parameter.rollup_block_height,
                        TransactionOrder::default(),
                        OrderHash::default(),
                    );
                    let rollup_metadata_model =
                        RollupMetadataModel::new(parameter.rollup_id, rollup_metadata);
                    rollup_metadata_model.put()?;
                    Self::sync_block(rollup_metadata_model);

                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }

    pub async fn sync_block(rollup_metadata_model: RollupMetadataModel) {
        tokio::spawn(async move {
            let transaction_order = rollup_metadata_model
                .rollup_metadata()
                .transaction_order()
                .value() as usize;
            let rollup_id = rollup_metadata_model.rollup_id();
            let rollup_block_height = rollup_metadata_model.rollup_metadata().block_height();

            // TODO: 1. make encrypted / raw transaction list
            let mut encrypted_transaction_list: Vec<EncryptedTransaction> =
                Vec::with_capacity(transaction_order);

            let mut raw_transaction_list: Vec<RawTransaction> =
                Vec::with_capacity(transaction_order);

            // TODO: change
            for order_index in 0..transaction_order as u64 {
                match EncryptedTransactionModel::get(
                    rollup_id,
                    rollup_block_height,
                    &TransactionOrder::new(order_index),
                ) {
                    Ok(encrypted_transaction) => {
                        encrypted_transaction_list
                            .push(encrypted_transaction.encrypted_transaction().clone());
                    }
                    // Todo: handling error
                    Err(_) => {}
                }

                match RawTransactionModel::get(
                    rollup_id,
                    rollup_block_height,
                    &TransactionOrder::new(order_index),
                ) {
                    Ok(raw_transaction) => {
                        raw_transaction_list.push(raw_transaction.raw_transaction().clone());
                    }
                    // TODO: handling error
                    Err(_) => {}
                }
            }

            // TODO: 2. make block commitment
            // get block_commitment option from config or cluster
            // change calculate logic
            // let seed = [0u8; 32];
            // let block_commitment: BlockCommitment = calculate_block_commitment(block, seed);
            // TODO: Check
            // block_commitment
            //     .put(rollup_id, rollup_block_height)
            //     .ok_or_trace();

            // TODO: 3. set proposer address
            let proposer_address = cluster.node_address();

            // TODO: 4. set timestamp
            let timestamp = Timestamp::new(chrono::Utc::now().timestamp().to_string());

            // TODO: 5. make block
            let block: Block = Block::new(
                rollup_block_height,
                EncryptedTransactionList::new(encrypted_transaction_list),
                RawTransactionList::new(raw_transaction_list),
                proposer_address.clone(),
                // Todo: change
                Signature::default(),
                timestamp,
                // Todo: change
                vec![0u8; 32].into(),
            );

            // TODO: 6. sign block (set signature)

            let block_model = BlockModel::new(rollup_id.clone(), block);

            block_model.put().unwrap();
        });
    }
}
