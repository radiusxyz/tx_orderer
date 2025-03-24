use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::try_join_all;
use skde::delay_encryption::{decrypt, SkdeParams};
use tokio::{
    sync::{Mutex, Notify, RwLock},
    time::sleep,
};

use crate::{
    client::distributed_key_generation::DistributedKeyGenerationClient,
    error::Error,
    types::{
        to_raw_tx, EncryptedTransaction, EthPlainData, EthRawTransaction, PlainData,
        RawTransaction, RawTransactionModel, RollupMetadata, SkdeEncryptedTransaction,
        TransactionData,
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
}

impl Decryptor {
    pub fn new(
        distributed_key_generation_client: DistributedKeyGenerationClient,
        skde_params: SkdeParams,
        latest_decryption_key_id: u64,
    ) -> Result<Arc<Self>, Error> {
        let decryptor = Arc::new(Self {
            inner: Arc::new(DecryptorInner {
                skde_params,
                latest_decryption_key_id: RwLock::new(latest_decryption_key_id),
                decryption_keys: Mutex::new(HashMap::new()),
                encrypted_transactions: Mutex::new(HashMap::new()),
                distributed_key_generation_client,
                notify: Notify::new(),
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

            let decryption_key_id_list = self
                .inner
                .encrypted_transactions
                .lock()
                .await
                .keys()
                .cloned()
                .collect::<Vec<_>>();

            for decryption_key_id in decryption_key_id_list {
                let decryption_key = match self
                    .inner
                    .decryption_keys
                    .lock()
                    .await
                    .get(&decryption_key_id)
                {
                    Some(decryption_key) => decryption_key.clone(),
                    None => {
                        tracing::info!("Fetching decryption key for key_id: {}", decryption_key_id);

                        match self
                            .inner
                            .distributed_key_generation_client
                            .get_decryption_key(decryption_key_id)
                            .await
                        {
                            Ok(get_decryption_key_response) => {
                                self.inner.decryption_keys.lock().await.insert(
                                    decryption_key_id,
                                    get_decryption_key_response.decryption_key.clone(),
                                );
                                get_decryption_key_response.decryption_key
                            }
                            Err(e) => {
                                tracing::error!("Failed to get decryption key: {:?}", e);
                                continue;
                            }
                        }
                    }
                };

                let encrypted_transactions = self
                    .inner
                    .encrypted_transactions
                    .lock()
                    .await
                    .remove(&decryption_key_id)
                    .unwrap_or_default();

                let mut decryption_handle_list = Vec::new();
                let decrypted_transaction_order_list: Arc<Mutex<Vec<(String, u64)>>> =
                    Arc::new(Mutex::new(Vec::new()));
                for (rollup_id, batch_number, transaction_order, encrypted_transaction) in
                    encrypted_transactions
                {
                    let skde_params = self.inner.skde_params.clone();
                    let decryption_key = decryption_key.clone();
                    let cloned_decrypted_transaction_order_list =
                        Arc::clone(&decrypted_transaction_order_list);
                    let decryption_handle = tokio::spawn(async move {
                        match decrypt_skde_transaction(
                            &skde_params,
                            &decryption_key,
                            &encrypted_transaction,
                        )
                        .await
                        {
                            Ok((raw_transaction, plain_data)) => {
                                tracing::info!(
                                    "Decrypted transaction: {:?}, plain_data: {:?}",
                                    raw_transaction,
                                    plain_data
                                );

                                let _ = RawTransactionModel::put(
                                    &rollup_id,
                                    batch_number,
                                    transaction_order,
                                    raw_transaction.clone(),
                                    false,
                                );

                                cloned_decrypted_transaction_order_list
                                    .lock()
                                    .await
                                    .push((rollup_id, transaction_order));
                            }
                            Err(e) => {
                                tracing::error!("Failed to decrypt transaction: {:?}", e);
                            }
                        }

                        ()
                    });
                    decryption_handle_list.push(decryption_handle);
                }

                let decrypted_transaction_order_per_rollup: HashMap<String, Vec<u64>> = {
                    let list = decrypted_transaction_order_list.lock().await;
                    let mut map = HashMap::new();

                    for (key, value) in list.iter() {
                        map.entry(key.clone()).or_insert_with(Vec::new).push(*value);
                    }

                    map
                };

                for (rollup_id, transaction_order_list) in decrypted_transaction_order_per_rollup {
                    let mut rollup_metadata =
                        RollupMetadata::get_mut(&rollup_id).expect("Failed to get rollup metadata");
                    rollup_metadata
                        .can_provide_transactions
                        .extend(transaction_order_list);
                    rollup_metadata
                        .update()
                        .expect("Failed to update rollup metadata");
                }

                let result_list = try_join_all(decryption_handle_list).await;
                if let Err(error) = result_list {
                    tracing::error!("Failed to join decryption tasks: {:?}", error);
                }
            }
        }
    }

    async fn process_to_get_decryption_key(&self) {
        loop {
            println!("stompesi - process_to_get_decryption_key");
            sleep(Duration::from_secs(1)).await;

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

                    tracing::info!("Decryption key fetched for key_id: {}", decryption_key_id);
                }

                Err(e) => {
                    tracing::error!("Failed to get decryption key: {:?}", e);
                }
            }
        }
    }

    pub async fn add_encrypted_transaction_to_decrypt(
        &self,
        rollup_id: String,
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
                            rollup_id,
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
