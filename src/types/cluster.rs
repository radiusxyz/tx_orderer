use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::context::SharedContext;
use skde::key_generation::PartialKey;
use tokio::sync::Mutex;

use super::prelude::*;
use crate::client::SequencerClient;

pub type SequencerIndex = usize;
pub type ClusterIdList = Vec<ClusterId>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterId(String);

impl From<String> for ClusterId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub struct Cluster {
    inner: Arc<ClusterInner>,
}

struct ClusterInner {
    cluster_id: ClusterId,
    node_address: Address,
    sequencer_rpc_client_list: SharedContext<Vec<(Address, SequencerClient)>>,

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
            sequencer_rpc_client_list: SharedContext::from(Vec::new()),
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
        let partial_keys_lock = self.inner.partial_keys.lock().await.clone();
        println!("stompesi - partial_keys_lock: {:?}", partial_keys_lock);

        let sorted_partial_key_list: Vec<PartialKey> = self
            .inner
            .sequencer_rpc_client_list
            .load()
            .as_ref()
            .iter()
            .map(|(address, _)| partial_keys_lock.get(address).cloned())
            .flatten()
            .collect();

        sorted_partial_key_list
    }

    pub async fn add_sequencer_rpc_client(
        &self,
        address: Address,
        sequencer_client: SequencerClient,
    ) {
        let mut sequencer_rpc_clients =
            self.inner.sequencer_rpc_client_list.load().as_ref().clone();
        sequencer_rpc_clients.push((address.clone(), sequencer_client));

        self.inner
            .sequencer_rpc_client_list
            .store(sequencer_rpc_clients);
    }

    // Todo: after remove, sort sequencer index
    pub fn remove_sequencer_rpc_client(&self, address: Address) {
        let mut sequencer_rpc_client_list =
            self.inner.sequencer_rpc_client_list.load().as_ref().clone();

        sequencer_rpc_client_list.retain(|(rpc_address, _)| rpc_address != &address);

        self.inner
            .sequencer_rpc_client_list
            .store(sequencer_rpc_client_list);
    }

    pub fn set_sequencer_rpc_client_list(
        &mut self,
        sequencer_rpc_clients: Vec<(Address, SequencerClient)>,
    ) {
        self.inner
            .sequencer_rpc_client_list
            .store(sequencer_rpc_clients)
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.inner.cluster_id
    }

    pub async fn get_other_sequencer_rpc_clients(&self) -> Vec<SequencerClient> {
        self.inner
            .sequencer_rpc_client_list
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
            .sequencer_rpc_client_list
            .load()
            .as_ref()
            .iter()
            .map(|(_, rpc_client)| rpc_client.clone())
            .collect()
    }

    pub async fn is_leader(&self, rollup_block_height: BlockHeight) -> bool {
        let sequencer_rpc_client_list_context = self.inner.sequencer_rpc_client_list.load();

        let sequencer_rpc_client_list = sequencer_rpc_client_list_context.as_ref();
        let leader_index = (rollup_block_height % sequencer_rpc_client_list.len() as BlockHeight)
            as SequencerIndex;

        sequencer_rpc_client_list
            .get(leader_index)
            .map(|(address, _)| address == self.node_address())
            .unwrap_or(false)
    }

    pub async fn get_leader_rpc_client(&self, rollup_block_height: BlockHeight) -> SequencerClient {
        let sequencer_rpc_client_list_context = self.inner.sequencer_rpc_client_list.load();

        let sequencer_rpc_client_list = sequencer_rpc_client_list_context.as_ref();
        let leader_index = (rollup_block_height % sequencer_rpc_client_list.len() as BlockHeight)
            as SequencerIndex;

        println!("jaemin - leader_index: {:?}", leader_index);

        let leader = sequencer_rpc_client_list.get(leader_index).unwrap();
        println!("jaemin - leader_address: {:?}", leader.0);

        leader.1.clone()
    }

    pub async fn get_follower_rpc_client_list(
        &self,
        rollup_block_height: BlockHeight,
    ) -> Vec<SequencerClient> {
        let sequencer_rpc_client_list_context = self.inner.sequencer_rpc_client_list.load();

        let sequencer_rpc_client_list = sequencer_rpc_client_list_context.as_ref();
        let leader_index = (rollup_block_height % sequencer_rpc_client_list.len() as BlockHeight)
            as SequencerIndex;

        sequencer_rpc_client_list
            .iter()
            .enumerate()
            .filter_map(|(sequencer_index, (address, rpc_client))| {
                (address != self.node_address() && sequencer_index != leader_index)
                    .then(|| rpc_client.clone())
            })
            .collect()
    }
}
