use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::context::SharedContext;
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

    // Todo: remove
    sequencer_indexes: Mutex<HashMap<SequencerIndex, Address>>,
    // Todo: change HashMap to Vec
    sequencer_rpc_clients: SharedContext<Vec<(Address, SequencerClient)>>,
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
            sequencer_rpc_clients: SharedContext::from(Vec::new()),
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
        address: Address,
        sequencer_client: SequencerClient,
    ) {
        let mut sequencer_rpc_clients = self.inner.sequencer_rpc_clients.load().as_ref().clone();
        let sequencer_index = sequencer_rpc_clients.len();
        sequencer_rpc_clients.push((address.clone(), sequencer_client));

        self.inner
            .sequencer_rpc_clients
            .store(sequencer_rpc_clients);

        let mut sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        sequencer_indexes_lock.insert(sequencer_index, address);
    }

    // Todo: after remove, sort sequencer index
    pub fn remove_sequencer_rpc_client(&self, address: Address) {
        let mut sequencer_rpc_clients = self.inner.sequencer_rpc_clients.load().as_ref().clone();

        sequencer_rpc_clients.retain(|(rpc_address, _)| rpc_address != &address);

        self.inner
            .sequencer_rpc_clients
            .store(sequencer_rpc_clients);
    }

    pub async fn set_sequencer_indexes(
        &mut self,
        sequencer_indexes: HashMap<SequencerIndex, Address>,
    ) {
        let mut sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;

        *sequencer_indexes_lock = sequencer_indexes;
    }

    pub fn set_sequencer_rpc_clients(
        &mut self,
        sequencer_rpc_clients: Vec<(Address, SequencerClient)>,
    ) {
        self.inner
            .sequencer_rpc_clients
            .store(sequencer_rpc_clients)
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.inner.cluster_id
    }

    pub async fn get_other_sequencer_rpc_clients(&self) -> Vec<SequencerClient> {
        self.inner
            .sequencer_rpc_clients
            .load()
            .as_ref()
            .iter()
            .filter_map(|(address, rpc_client)| {
                (address != &self.inner.node_address).then(|| rpc_client.clone())
            })
            .collect()
    }

    pub async fn sequencer_rpc_clients(&self) -> Vec<SequencerClient> {
        self.inner
            .sequencer_rpc_clients
            .load()
            .as_ref()
            .iter()
            .map(|(_, rpc_client)| rpc_client.clone())
            .collect()
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

        println!("jaemin - sequencer_indexes: {:?}", sequencer_indexes_lock);

        println!("jaemin - leader_index: {:?}", leader_index);

        let leader_address = sequencer_indexes_lock.get(&leader_index).unwrap();
        println!("jaemin - leader_address: {:?}", leader_address);

        let leader_rpc_client = self
            .inner
            .sequencer_rpc_clients
            .load()
            .as_ref()
            .iter()
            .find(|(address, _)| address == leader_address)
            .map(|(_, sequencer_client)| sequencer_client.clone())
            .unwrap();

        leader_rpc_client
    }

    pub async fn get_follower_rpc_client_list(
        &self,
        rollup_block_height: BlockHeight,
    ) -> Vec<SequencerClient> {
        let sequencer_indexes_lock = self.inner.sequencer_indexes.lock().await;
        let leader_index =
            (rollup_block_height % sequencer_indexes_lock.len() as BlockHeight) as SequencerIndex;

        let leader_address = sequencer_indexes_lock.get(&leader_index).unwrap();

        // TODO
        let follower_rpc_client_list = self
            .inner
            .sequencer_rpc_clients
            .load()
            .as_ref()
            .iter()
            .filter_map(|(address, rpc_client)| {
                (address != self.node_address() && address != leader_address)
                    .then(|| rpc_client.clone())
            })
            .collect::<Vec<SequencerClient>>();

        follower_rpc_client_list
    }
}
