use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::json_rpc::RpcClient;
use tokio::sync::Mutex;

use super::prelude::*;

pub type SequencerIndex = usize;
pub type ClusterId = String;
pub type ClusterIdList = Vec<ClusterId>;

pub struct Cluster {
    inner: Arc<ClusterInner>,
}

struct ClusterInner {
    cluster_id: ClusterId,

    node_address: Address,
    sequencer_rpc_clients: Mutex<HashMap<(SequencerIndex, Address), RpcClient>>,
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
            sequencer_rpc_clients: Mutex::new(HashMap::new()),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn set_sequencer_rpc_clients(
        &mut self,
        sequencer_rpc_clients: HashMap<(SequencerIndex, Address), RpcClient>,
    ) {
        let mut sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        *sequencer_rpc_clients_lock = sequencer_rpc_clients;
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.inner.cluster_id
    }

    pub async fn get_other_sequencer_rpc_clients(&self) -> Vec<RpcClient> {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        sequencer_rpc_clients_lock
            .iter()
            .filter_map(|((_, address), rpc_client)| {
                if address != &self.inner.node_address {
                    Some(rpc_client.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<RpcClient>>()
    }

    pub async fn is_leader(&self, rollup_block_height: BlockHeight) -> bool {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        let leader_index = rollup_block_height % sequencer_rpc_clients_lock.len() as BlockHeight;

        let leader_address = sequencer_rpc_clients_lock
            .keys()
            .find(|(index, _)| *index == leader_index as usize)
            .unwrap();

        leader_address.1 == self.inner.node_address
    }

    pub async fn get_leader_rpc_client(&self, rollup_block_height: BlockHeight) -> RpcClient {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        let leader_index = rollup_block_height % sequencer_rpc_clients_lock.len() as BlockHeight;

        let leader_address = sequencer_rpc_clients_lock
            .keys()
            .find(|(index, _)| *index == leader_index as usize)
            .unwrap();

        let leader_rpc_client = sequencer_rpc_clients_lock
            .get(leader_address)
            .cloned()
            .unwrap();

        leader_rpc_client
    }

    pub async fn get_follower_rpc_client_list(
        &self,
        rollup_block_height: BlockHeight,
    ) -> Vec<RpcClient> {
        let sequencer_rpc_clients_lock = self.inner.sequencer_rpc_clients.lock().await;

        let leader_index = rollup_block_height % sequencer_rpc_clients_lock.len() as BlockHeight;

        let follower_rpc_client_list = sequencer_rpc_clients_lock
            .iter()
            .filter_map(|((index, address), rpc_client)| {
                if *index == leader_index as usize && address != &self.inner.node_address {
                    Some(rpc_client.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<RpcClient>>();

        follower_rpc_client_list
    }
}
