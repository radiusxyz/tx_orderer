use std::collections::BTreeSet;

use radius_sdk::signature::Address;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionList {
    pub rollup_id: String,
    pub executor_address: Address,
    pub platform_block_height: u64,

    pub current_leader_tx_orderer_address: Address,
    pub next_leader_tx_orderer_address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionListResponse {
    pub raw_transaction_list: Vec<String>,
}
impl GetRawTransactionList {
    async fn get_raw_transaction_list(self) -> Result<GetRawTransactionListResponse, RpcError> {
        let mut rollup_metadata = RollupMetadata::get(&self.rollup_id)?;

        let start_batch_number = rollup_metadata.provided_batch_number;
        let end_batch_number = rollup_metadata.can_provide_batch_number;
        let start_transaction_order = rollup_metadata.provided_transaction_order + 1;

        let mut raw_transaction_list = Vec::new();

        fn extract_raw_transactions(batch: Batch) -> Vec<String> {
            batch
                .raw_transaction_list
                .into_iter()
                .map(|transaction| match transaction {
                    RawTransaction::Eth(EthRawTransaction(data)) => data,
                    RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
                })
                .collect()
        }

        fn get_last_valid_transaction(can_provide_transactions: &BTreeSet<u64>, start: u64) -> u64 {
            let mut last_valid = start;
            for &tx in can_provide_transactions {
                if tx == last_valid {
                    last_valid += 1;
                } else {
                }
            }
            last_valid
        }

        fn fetch_and_append_transactions(
            rollup_id: &str,
            batch_number: u64,
            start_transaction_order: u64,
            last_valid_transaction_order: u64,
            raw_transaction_list: &mut Vec<String>,
        ) -> Result<(), RpcError> {
            for transaction_order in start_transaction_order..last_valid_transaction_order {
                let (raw_transaction, _) =
                    RawTransactionModel::get(rollup_id, batch_number, transaction_order)?;
                let raw_transaction = match raw_transaction {
                    RawTransaction::Eth(EthRawTransaction(data)) => data,
                    RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
                };
                raw_transaction_list.push(raw_transaction);
            }
            Ok(())
        }

        let last_valid_transaction_order;
        if start_batch_number == end_batch_number {
            last_valid_transaction_order = get_last_valid_transaction(
                &rollup_metadata.can_provide_transactions,
                start_transaction_order,
            );

            fetch_and_append_transactions(
                &self.rollup_id,
                start_batch_number,
                start_transaction_order,
                last_valid_transaction_order,
                &mut raw_transaction_list,
            )?;
        } else if start_batch_number < end_batch_number {
            let batch: Batch = Batch::get(&self.rollup_id, start_batch_number)?;

            for transaction_order in
                start_transaction_order..batch.raw_transaction_list.len() as u64
            {
                let raw_transaction = match &batch.raw_transaction_list[transaction_order as usize]
                {
                    RawTransaction::Eth(EthRawTransaction(data)) => data,
                    RawTransaction::EthBundle(EthRawBundleTransaction(data)) => data,
                };

                raw_transaction_list.push(raw_transaction.clone());
            }

            for batch_number in (start_batch_number + 1)..end_batch_number {
                let batch = Batch::get(&self.rollup_id, batch_number)?;
                raw_transaction_list.extend(extract_raw_transactions(batch));
            }

            last_valid_transaction_order =
                get_last_valid_transaction(&rollup_metadata.can_provide_transactions, 0);

            fetch_and_append_transactions(
                &self.rollup_id,
                start_batch_number,
                0,
                last_valid_transaction_order,
                &mut raw_transaction_list,
            )?;
        } else {
            return Err(Error::InvalidBatchNumber.into());
        }

        rollup_metadata.provided_batch_number = end_batch_number;
        rollup_metadata.provided_transaction_order = last_valid_transaction_order;
        rollup_metadata.put(&self.rollup_id)?;

        Ok(GetRawTransactionListResponse {
            raw_transaction_list,
        })
    }
}

impl RpcParameter<AppState> for GetRawTransactionList {
    type Response = GetRawTransactionListResponse;

    fn method() -> &'static str {
        "get_raw_transaction_list"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        self.get_raw_transaction_list().await
    }
}

#[cfg(test)]
mod tests {
    use radius_sdk::{kvstore::KvStoreBuilder, signature::ChainType};

    use super::*;

    fn mock_rollup_metadata(
        provided_batch: u64,
        provided_tx_order: u64,
        can_provide_batch: u64,
        can_provide_tx_list: &[u64],
    ) -> RollupMetadata {
        RollupMetadata {
            batch_number: 0,
            transaction_order: 0,
            max_gas_limit: 100000,
            current_gas: 0,
            cluster_id: "test-cluster".to_string(),
            provided_batch_number: provided_batch,
            provided_transaction_order: provided_tx_order,
            can_provide_batch_number: can_provide_batch,
            can_provide_transactions: can_provide_tx_list.iter().cloned().collect(),
        }
    }

    fn get_address() -> Address {
        Address::from_str(
            ChainType::Ethereum,
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
        )
        .unwrap()
    }

    fn mock_batch_with_raw_txs(tx_data: &[&str]) -> Batch {
        let mut batch = Batch::new(
            0,
            vec![],
            vec![],
            BatchCommitment::default(),
            get_address(),
            Signature::from(Vec::new()),
        );

        batch.raw_transaction_list = tx_data
            .iter()
            .map(|&data| RawTransaction::Eth(EthRawTransaction(data.to_string())))
            .collect();
        batch
    }

    fn setup_test_environment() {
        let database_path = "./test_db";
        let kv_store = KvStoreBuilder::default()
            .set_default_lock_timeout(5000)
            .set_txn_lock_timeout(5000)
            .build(database_path)
            .map_err(Error::Database)
            .unwrap();
        kv_store.init();
        tracing::info!("Database initialized at {:?}", database_path);
    }

    fn remove_test_environment() {
        let database_path = "./test_db";
        std::fs::remove_dir_all(database_path).unwrap_or_else(|_| {
            tracing::warn!("Failed to remove test database at {:?}", database_path);
        });
    }

    #[tokio::test]
    async fn test_handler_same_batch_with_continuous_txs() {
        setup_test_environment();

        let rollup_id = "test-rollup";

        let provided_batch_number = 1;
        let provided_tx_order = 1;

        let can_provide_batch_number = 1;
        let can_provide_tx_list = [2, 3, 4];

        let rollup_metadata = mock_rollup_metadata(
            provided_batch_number,
            provided_tx_order,
            can_provide_batch_number,
            &can_provide_tx_list,
        );
        let _ = rollup_metadata.put(rollup_id);

        for transaction_order in can_provide_tx_list {
            let raw_transaction_str =
                format!("tx{}_{}", can_provide_batch_number, transaction_order);
            let raw_transaction =
                RawTransaction::Eth(EthRawTransaction(raw_transaction_str.to_string()));

            let _ = RawTransactionModel::put(
                rollup_id,
                provided_batch_number,
                transaction_order,
                raw_transaction,
                true,
            );
        }

        let get_raw_transaction_list = GetRawTransactionList {
            rollup_id: rollup_id.to_string(),
            executor_address: get_address(),
            platform_block_height: 0,
            current_leader_tx_orderer_address: get_address(),
            next_leader_tx_orderer_address: get_address(),
        };

        let result = get_raw_transaction_list
            .get_raw_transaction_list()
            .await
            .unwrap();

        assert_eq!(result.raw_transaction_list, vec!["tx1_2", "tx1_3", "tx1_4"]);

        remove_test_environment();
    }

    #[tokio::test]
    async fn test_handler_multiple_batches_with_continuous_txs() {
        setup_test_environment();

        let rollup_id = "test-rollup";

        let provided_batch_number = 1;
        let provided_tx_order = 1;

        let can_provide_batch_number = 3;
        let can_provide_tx_list = [0, 1];

        let rollup_metadata = mock_rollup_metadata(
            provided_batch_number,
            provided_tx_order,
            can_provide_batch_number,
            &can_provide_tx_list,
        );
        let _ = rollup_metadata.put(rollup_id);

        println!("Rollup Metadata: {:?}", rollup_metadata);

        for batch_number in provided_batch_number..can_provide_batch_number {
            let txs = vec![
                format!("tx{}_0", batch_number),
                format!("tx{}_1", batch_number),
            ];
            let batch =
                mock_batch_with_raw_txs(&txs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            let _ = batch.put(rollup_id, batch_number);
        }

        for transaction_order in can_provide_tx_list {
            let raw_transaction_str =
                format!("tx{}_{}", can_provide_batch_number, transaction_order);
            let raw_transaction =
                RawTransaction::Eth(EthRawTransaction(raw_transaction_str.to_string()));

            let _ = RawTransactionModel::put(
                rollup_id,
                provided_batch_number,
                transaction_order,
                raw_transaction,
                true,
            );
        }

        let get_raw_transaction_list = GetRawTransactionList {
            rollup_id: rollup_id.to_string(),
            executor_address: get_address(),
            platform_block_height: 0,
            current_leader_tx_orderer_address: get_address(),
            next_leader_tx_orderer_address: get_address(),
        };

        let result = get_raw_transaction_list
            .get_raw_transaction_list()
            .await
            .unwrap();

        assert_eq!(
            result.raw_transaction_list,
            vec!["tx2_0", "tx2_1", "tx3_0", "tx3_1"]
        );
        remove_test_environment();
    }

    #[tokio::test]
    async fn test_handler_with_non_continuous_txs() {
        setup_test_environment();

        let rollup_id = "test-rollup";

        let provided_batch_number = 1;
        let provided_tx_order = 0;

        let can_provide_batch_number = 1;
        let can_provide_tx_list = [1, 2, 4, 5];

        let rollup_metadata = mock_rollup_metadata(
            provided_batch_number,
            provided_tx_order,
            can_provide_batch_number,
            &can_provide_tx_list,
        );
        let _ = rollup_metadata.put(rollup_id);

        for transaction_order in can_provide_tx_list {
            let raw_transaction_str =
                format!("tx{}_{}", can_provide_batch_number, transaction_order);
            let raw_transaction =
                RawTransaction::Eth(EthRawTransaction(raw_transaction_str.to_string()));

            let _ = RawTransactionModel::put(
                rollup_id,
                provided_batch_number,
                transaction_order,
                raw_transaction,
                true,
            );
        }

        let get_raw_transaction_list = GetRawTransactionList {
            rollup_id: rollup_id.to_string(),
            executor_address: get_address(),
            platform_block_height: 0,
            current_leader_tx_orderer_address: get_address(),
            next_leader_tx_orderer_address: get_address(),
        };

        let result = get_raw_transaction_list
            .get_raw_transaction_list()
            .await
            .unwrap();

        assert_eq!(result.raw_transaction_list, vec!["tx1_1", "tx1_2"]);
        remove_test_environment();
    }
}
