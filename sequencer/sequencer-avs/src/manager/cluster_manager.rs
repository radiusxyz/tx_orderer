use std::{
    collections::HashMap,
    sync::{Arc, Once},
};

use database::Database;
use json_rpc::RpcClient;
use ssal::avs::types::Address;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

static INIT: Once = Once::new();

use crate::{
    error::Error,
    state::AppState,
    types::{ClusterMetadata, OrderCommitment, SequencerList, UserTransaction},
};

pub struct ClusterManager {
    inner: Arc<Cluster>,
}

struct Cluster {
    context: AppState,
    my_address: Address,
    sequencer_map: Mutex<HashMap<Address, Option<RpcClient>>>,
    data_queue: Sender<DataType>,
}

unsafe impl Send for ClusterManager {}

unsafe impl Sync for ClusterManager {}

impl Clone for ClusterManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl From<Cluster> for ClusterManager {
    fn from(value: Cluster) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl ClusterManager {
    pub fn init(context: AppState) -> Result<Self, Error> {
        let (sender, receiver) = channel::<DataType>(1024);

        let cluster_manager: ClusterManager = Cluster {
            context,
            my_address: context.ssal_client().address(),
            sequencer_map: Mutex::new(HashMap::<Address, Option<RpcClient>>::new()),
            data_queue: sender,
        }
        .into();
        cluster_manager.init_listener(receiver);

        Ok(cluster_manager)
    }

    pub async fn sync_user_transaction(
        &self,
        user_transaction: UserTransaction,
        order_commitment: OrderCommitment,
    ) {
        self.inner
            .data_queue
            .send(DataType::SyncUserTransaction(
                user_transaction,
                order_commitment,
            ))
            .await;
    }

    pub fn sync_cluster_metadata(&self) {}

    pub fn update_cluster(&self) {}

    fn init_listener(&self, mut receiver: Receiver<DataType>) {
        let cluster_manager = self.clone();

        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                match data {
                    DataType::SyncUserTransaction(user_transaction, order_commitment) => {}
                    DataType::SyncClusterMetadata() => {}
                }
            }
        });
    }
}

enum DataType {
    SyncUserTransaction(UserTransaction, OrderCommitment),
    SyncClusterMetadata(),
    // ClusterMetadata(ClusterMetadata),
}
