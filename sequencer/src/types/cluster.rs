use std::{collections::HashMap, sync::Arc};

use skde::key_generation::PartialKey;
use tokio::sync::Mutex;

use super::prelude::*;
use crate::client::SequencerClient;

pub type SequencerIndex = usize;
pub type ClusterId = String;
pub type ClusterIdList = Vec<ClusterId>;

pub struct Cluster {
    inner: Arc<ClusterInner>,
}

struct ClusterInner {
    cluster_id: ClusterId,

    node_address: Address,

    sequencer_indexes: Mutex<HashMap<SequencerIndex, Address>>,
    sequencer_rpc_clients: Mutex<HashMap<Address, SequencerClient>>,
    partial_keys: Mutex<HashMap<Address, PartialKey>>,
}

unsafe impl Send for Cluster {}

unsafe impl Sync for Cluster {}

impl Clone for Cluster {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Cluster {
    pub fn new(cluster_id: ClusterId, node_address: Address) -> Self {
        let inner = ClusterInner {
            cluster_id,
            node_address,
            sequencer_indexes: Mutex::new(HashMap::new()),
            sequencer_rpc_clients: Mutex::new(HashMap::new()),
            partial_keys: Mutex::new(HashMap::new()),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn node_address(&self) -> &Address {
        &self.inner.node_address
    }

    pub async fn add_partial_key(&self, address: Address, partial_key: PartialKey) {
        let mut partial_keys_lock = self.inner.partial_keys.lock().await;
        partial_keys_lock.insert(address, partial_key);
    }

    pub async fn get_partial_key_list(&self) -> Vec<PartialKey> {
        // TODO
        let sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        let partial_keys_lock = self.inner.partial_keys.lock().await;

        println!(
            "stompesi - sequencer_indexes_lock: {:?}",
            sequencer_indexes_lock
        );
        println!("stompesi - partial_keys_lock: {:?}", partial_keys_lock);

        let sorted_partial_key_list: Vec<PartialKey> = sequencer_indexes_lock
            .iter()
            .map(|(_, address)| partial_keys_lock.get(address).cloned())
            .flatten()
            .collect();

        sorted_partial_key_list
    }

    pub async fn add_sequencer_rpc_client(
        &self,
        sequencer_index: SequencerIndex,
        address: Address,
        sequencer_client: SequencerClient,
    ) {
        let mut sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        sequencer_indexes_lock.insert(sequencer_index, address.clone());

        let mut sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;
        sequencer_rpc_clients_lock.insert(address, sequencer_client);
    }

    pub async fn remove_sequencer_rpc_client(&self, address: Address) {
        let mut sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        sequencer_rpc_clients_lock.retain(|sequencer_address, _| sequencer_address != &address);
    }

    pub async fn set_sequencer_indexes(
        &mut self,
        sequencer_indexes: HashMap<SequencerIndex, Address>,
    ) {
        let mut sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;

        *sequencer_indexes_lock = sequencer_indexes;
    }

    pub async fn set_sequencer_rpc_clients(
        &mut self,
        sequencer_rpc_clients: HashMap<Address, SequencerClient>,
    ) {
        let mut sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        *sequencer_rpc_clients_lock = sequencer_rpc_clients;
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.inner.cluster_id
    }

    pub async fn get_other_sequencer_rpc_clients(&self) -> Vec<SequencerClient> {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        sequencer_rpc_clients_lock
            .iter()
            .filter_map(|(address, rpc_client)| {
                if address != &self.inner.node_address {
                    Some(rpc_client.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<SequencerClient>>()
    }

    pub async fn sequencer_rpc_clients(&self) -> Vec<SequencerClient> {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        // TODO
        sequencer_rpc_clients_lock
            .iter()
            .map(|(_, rpc_client)| rpc_client.clone())
            .collect::<Vec<SequencerClient>>()
    }

    pub async fn is_leader(&self, rollup_block_height: BlockHeight) -> bool {
        let sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;

        let leader_index =
            (rollup_block_height % sequencer_indexes_lock.len() as BlockHeight) as SequencerIndex;

        let leader_address = sequencer_indexes_lock.get(&leader_index).unwrap();

        leader_address == self.node_address()
    }

    pub async fn get_leader_rpc_client(&self, rollup_block_height: BlockHeight) -> SequencerClient {
        let sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        let leader_index =
            (rollup_block_height % sequencer_indexes_lock.len() as BlockHeight) as SequencerIndex;

        let leader_address = sequencer_indexes_lock.get(&leader_index).unwrap();

        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        let leader_rpc_client = sequencer_rpc_clients_lock
            .get(leader_address)
            .cloned()
            .unwrap();

        leader_rpc_client
    }

    pub async fn get_follower_rpc_client_list(
        &self,
        rollup_block_height: BlockHeight,
    ) -> Vec<SequencerClient> {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;
        let sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        let leader_index =
            (rollup_block_height % sequencer_indexes_lock.len() as BlockHeight) as SequencerIndex;

        let leader_address = sequencer_indexes_lock.get(&leader_index).unwrap();

        // TODO
        let follower_rpc_client_list = sequencer_rpc_clients_lock
            .iter()
            .filter_map(|(address, rpc_client)| {
                if address != self.node_address() && address != leader_address {
                    Some(rpc_client.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<SequencerClient>>();

        follower_rpc_client_list
    }
}
