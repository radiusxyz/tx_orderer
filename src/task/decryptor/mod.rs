use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::try_join_all;
use radius_sdk::json_rpc::client::{Id, RpcClient};
use skde::delay_encryption::{decrypt, SkdeParams};
use tokio::{
    sync::{Mutex, Notify, RwLock},
    time::sleep,
};

use crate::{
    client::distributed_key_generation::DistributedKeyGenerationClient,
    error::{self, Error},
    types::{
        to_raw_tx, CanProvideTransactionInfo, EncryptedTransaction, EthPlainData,
        EthRawTransaction, PlainData, RawTransaction, RawTransactionModel, RollupId,
        SkdeEncryptedTransaction, TransactionData,
    },
};

pub struct Decryptor {
    inner: Arc<DecryptorInner>,
}

struct DecryptorInner {
    skde_params: SkdeParams,
    latest_decryption_key_id: RwLock<u64>,
    decryption_keys: Mutex<HashMap<u64, String>>,
    distributed_key_generation_client: DistributedKeyGenerationClient,
    encrypted_transactions: Mutex<HashMap<u64, Vec<(String, u64, u64, SkdeEncryptedTransaction)>>>,
    notify: Notify,
    rpc_client: Arc<RpcClient>,
    builder_rpc_url: Option<String>,
}

impl Decryptor {
    pub fn new(
        distributed_key_generation_client: DistributedKeyGenerationClient,
        skde_params: SkdeParams,
        latest_decryption_key_id: u64,
        builder_rpc_url: Option<String>,
    ) -> Result<Arc<Self>, Error> {
        let decryptor = Arc::new(Self {
            inner: Arc::new(DecryptorInner {
                skde_params,
                latest_decryption_key_id: RwLock::new(latest_decryption_key_id),
                decryption_keys: Mutex::new(HashMap::new()),
                encrypted_transactions: Mutex::new(HashMap::new()),
                distributed_key_generation_client,
                notify: Notify::new(),
                rpc_client: RpcClient::new().map_err(error::Error::RpcClient)?,
                builder_rpc_url,
            }),
        });

        Ok(decryptor)
    }

    pub async fn start(decryptor: Arc<Self>) {
        let cloned_decryptor = Arc::clone(&decryptor);
        tokio::spawn(async move { cloned_decryptor.process_to_get_decryption_key().await });

        let cloned_decryptor = Arc::clone(&decryptor);
        tokio::spawn(async move { cloned_decryptor.process_to_decrypt().await });
    }

    async fn process_to_decrypt(&self) {
        loop {
            self.inner.notify.notified().await;

            let decryption_key_id_list: Vec<_> = {
                let encrypted_transactions = self.inner.encrypted_transactions.lock().await;
                encrypted_transactions.keys().cloned().collect()
            };

            let current_can_decryption_keys = self.inner.decryption_keys.lock().await;

            let filtered_key_ids: Vec<_> = decryption_key_id_list
                .into_iter()
                .filter(|key_id| current_can_decryption_keys.contains_key(key_id))
                .collect();

            for decryption_key_id in filtered_key_ids {
                if let Some(decryption_key) = current_can_decryption_keys.get(&decryption_key_id) {
                    let encrypted_transactions = {
                        let mut encrypted_transactions =
                            self.inner.encrypted_transactions.lock().await;
                        encrypted_transactions
                            .remove(&decryption_key_id)
                            .unwrap_or_default()
                    };

                    let mut decryption_handle_list = Vec::new();
                    let decrypted_transaction_order_list: Arc<Mutex<Vec<(String, u64, u64)>>> =
                        Arc::new(Mutex::new(Vec::new()));
                    for (rollup_id, batch_number, transaction_order, encrypted_transaction) in
                        encrypted_transactions
                    {
                        let skde_params = self.inner.skde_params.clone();
                        let decryption_key = decryption_key.clone();
                        let cloned_decrypted_transaction_order_list =
                            Arc::clone(&decrypted_transaction_order_list);

                        let cloned_builder_rpc_url = self.inner.builder_rpc_url.clone();
                        let cloned_rpc_client = Arc::clone(&self.inner.rpc_client);

                        let decryption_handle = tokio::spawn(async move {
                            match decrypt_skde_transaction(
                                &skde_params,
                                &decryption_key,
                                &encrypted_transaction,
                            )
                            .await
                            {
                                Ok((raw_transaction, _plain_data)) => {
                                    let raw_transaction_hash = encrypted_transaction
                                        .transaction_data
                                        .raw_transaction_hash();

                                    let _ = RawTransactionModel::put_with_transaction_hash(
                                        &rollup_id,
                                        &raw_transaction_hash,
                                        raw_transaction.clone(),
                                        false,
                                    )
                                    .map_err(|error| {
                                        tracing::error!(
                                            "Failed to put raw transaction with hash: {:?}",
                                            error
                                        );
                                        Error::Database(error)
                                    });

                                    let _ = RawTransactionModel::put(
                                        &rollup_id,
                                        batch_number,
                                        transaction_order,
                                        raw_transaction.clone(),
                                        false,
                                    )
                                    .map_err(|error| {
                                        tracing::error!(
                                            "Failed to put raw transaction: {:?}",
                                            error
                                        );
                                        Error::Database(error)
                                    });

                                    cloned_decrypted_transaction_order_list.lock().await.push((
                                        rollup_id,
                                        batch_number,
                                        transaction_order,
                                    ));

                                    if cloned_builder_rpc_url.is_some() {
                                        let params = serde_json::json!([
                                            raw_transaction,
                                            batch_number,
                                            transaction_order
                                        ]);

                                        let _: String = cloned_rpc_client
                                            .request(
                                                &cloned_builder_rpc_url.clone().unwrap(),
                                                "eth_sendRawTransaction",
                                                &params,
                                                Id::Null,
                                            )
                                            .await
                                            .map_err(|error| {
                                                tracing::error!(
                                                    "Failed to send raw transaction: {:?}",
                                                    error
                                                );
                                                Error::RpcClient(error)
                                            })
                                            .unwrap();
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to decrypt transaction: {:?}", e);
                                }
                            }

                            ()
                        });
                        decryption_handle_list.push(decryption_handle);
                    }

                    let result_list = try_join_all(decryption_handle_list).await;

                    if let Err(error) = result_list {
                        tracing::error!("Failed to join decryption tasks: {:?}", error);
                    }

                    let transaction_order_per_rollup = {
                        let list = decrypted_transaction_order_list.lock().await;
                        let mut transaction_order_per_rollup = HashMap::new();

                        for (rollup_id, batch_number, transaction_order) in list.iter() {
                            if transaction_order_per_rollup.contains_key(rollup_id) == false {
                                transaction_order_per_rollup
                                    .insert(rollup_id.clone(), HashMap::new());
                            }

                            let transaction_orders_per_batch =
                                transaction_order_per_rollup.get_mut(rollup_id).unwrap();

                            if transaction_orders_per_batch.contains_key(batch_number) == false {
                                transaction_orders_per_batch.insert(*batch_number, Vec::new());
                            }

                            transaction_orders_per_batch
                                .get_mut(batch_number)
                                .unwrap()
                                .push(*transaction_order);
                        }

                        transaction_order_per_rollup
                    };

                    for (rollup_id, transaction_orders_per_batch) in transaction_order_per_rollup {
                        for (batch_number, transaction_order_list) in
                            transaction_orders_per_batch.into_iter()
                        {
                            CanProvideTransactionInfo::add_can_provide_transaction_orders(
                                &rollup_id,
                                batch_number,
                                transaction_order_list,
                            )
                            .expect("Failed to add can provide transaction orders");
                        }
                    }
                }
            }
        }
    }

    async fn process_to_get_decryption_key(&self) {
        loop {
            sleep(Duration::from_millis(500)).await;

            let decryption_key_id = *self.inner.latest_decryption_key_id.read().await;

            match self
                .inner
                .distributed_key_generation_client
                .get_decryption_key(decryption_key_id)
                .await
            {
                Ok(get_decryption_key_response) => {
                    self.inner.decryption_keys.lock().await.insert(
                        decryption_key_id,
                        get_decryption_key_response.decryption_key,
                    );

                    let mut latest_decryption_key_id =
                        self.inner.latest_decryption_key_id.write().await;
                    *latest_decryption_key_id = decryption_key_id + 1;

                    self.inner.notify.notify_one();
                }

                Err(_error) => {}
            }
        }
    }

    pub async fn add_encrypted_transaction_to_decrypt(
        &self,
        rollup_id: RollupId,
        batch_number: u64,
        transaction_order: u64,
        encrypted_transaction: EncryptedTransaction,
    ) -> Result<(), Error> {
        {
            match encrypted_transaction {
                EncryptedTransaction::Skde(encrypted_transaction) => {
                    let mut encrypted_transactions = self.inner.encrypted_transactions.lock().await;
                    encrypted_transactions
                        .entry(encrypted_transaction.key_id)
                        .or_default()
                        .push((
                            rollup_id.clone(),
                            batch_number,
                            transaction_order,
                            encrypted_transaction,
                        ));
                }
            }
        }

        self.inner.notify.notify_one();

        Ok(())
    }
}

async fn decrypt_skde_transaction(
    skde_params: &SkdeParams,
    decryption_key: &str,
    skde_encrypted_transaction: &SkdeEncryptedTransaction,
) -> Result<(RawTransaction, PlainData), Error> {
    let decryption_key_id = skde_encrypted_transaction.key_id;

    match &skde_encrypted_transaction.transaction_data {
        TransactionData::Eth(transaction_data) => {
            let encrypted_data = transaction_data.encrypted_data.clone();

            let decrypted_data = decrypt(&skde_params, encrypted_data.as_ref(), &decryption_key)
                .map_err(|e| {
                    tracing::error!(
                        "Decryption failed for key_id: {}: {:?}",
                        decryption_key_id,
                        e
                    );
                    Error::Decryption
                })?;

            let eth_plain_data: EthPlainData =
                serde_json::from_str(&decrypted_data).map_err(|e| {
                    tracing::error!("Failed to parse decrypted data: {:?}", e);
                    Error::Deserialize
                })?;

            let rollup_transaction = transaction_data
                .open_data
                .convert_to_rollup_transaction(&eth_plain_data);

            let eth_raw_transaction = EthRawTransaction::from(to_raw_tx(rollup_transaction));
            let raw_transaction = RawTransaction::from(eth_raw_transaction);

            Ok((raw_transaction, PlainData::from(eth_plain_data)))
        }
        TransactionData::EthBundle(_data) => {
            tracing::warn!("EthBundle transactions are not yet supported.");
            unimplemented!()
        }
    }
}
