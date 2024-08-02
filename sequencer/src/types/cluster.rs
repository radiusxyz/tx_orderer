use std::{collections::HashMap, hash::Hash, sync::Arc};

use radius_sequencer_sdk::json_rpc::RpcClient;
use ssal::avs::LivenessClient;
use tokio::sync::Mutex;

use super::prelude::*;

pub type ProposerSetId = String;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClusterType {
    Local,
    EigenLayer,
}

struct ClusterInner {
    rollup_id: String,

    leader_address: Mutex<Address>,
    sequencer_rpc_clients: Mutex<HashMap<Address, RpcClient>>,
}

pub struct RollupCluster {
    inner: Arc<ClusterInner>,
}

unsafe impl Send for RollupCluster {}

unsafe impl Sync for RollupCluster {}

impl Clone for RollupCluster {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl RollupCluster {
    pub fn new(rollup_id: RollupId) -> Self {
        let inner = ClusterInner {
            rollup_id,
            leader_address: Mutex::new(Address::default()),
            sequencer_rpc_clients: Mutex::new(HashMap::new()),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn rollup_id(&self) -> &String {
        &self.inner.rollup_id
    }

    pub async fn get_leader_rpc_client(&self) -> RpcClient {
        let leader_address_lock = self.inner.leader_address.lock().await;
        let sequencers_lock = self.inner.sequencer_rpc_clients.lock().await;

        sequencers_lock.get(&*leader_address_lock).cloned().unwrap()
    }

    pub async fn get_follower_rpc_client_list(&self) -> Vec<RpcClient> {
        let leader_address_lock = self.inner.leader_address.lock().await;
        let sequencers_lock = self.inner.sequencer_rpc_clients.lock().await;

        sequencers_lock
            .iter()
            .filter(|(address, _)| **address != *leader_address_lock)
            .map(|(_, rpc_client)| rpc_client.clone())
            .collect()
    }

    pub async fn set_leader_address(&mut self, address: Address) {
        let mut leader_address_lock = self.inner.leader_address.lock().await;

        *leader_address_lock = address;
    }

    pub async fn set_sequencer_rpc_clients(&mut self, rpc_clients: HashMap<Address, RpcClient>) {
        let mut sequencers_lock = self.inner.sequencer_rpc_clients.lock().await;

        *sequencers_lock = rpc_clients;
    }

    // pub fn leader(
    //     &self,
    //     liveness_block_height: &BlockHeight,
    //     rollup_block_height: &BlockHeight,
    // ) -> &RpcClient {
    //     &self
    //         .inner
    //         .sequencer_list
    //         .get(*rollup_block_height as usize % self.inner.sequencer_list.len())
    //         .unwrap()
    // }

    // pub fn followers(&self) -> Vec<&RpcClient> {
    //     let exclude_index = *rollup_block_height as usize % self.inner.sequencer_list.len();

    //     self.inner
    //         .sequencer_list
    //         .iter()
    //         .enumerate()
    //         .filter(|(i, _)| *i != exclude_index)
    //         .map(|(_, sequencer)| sequencer)
    //         .collect::<Vec<&RpcClient>>()
    // }

    // async fn update_sequencer_list(&mut self, liveness_block_height: BlockHeight) {
    //     let mut sequencer_list = SequencerList::get(liveness_block_height)?.into_inner();

    //     &self.inner.sequencer_list = self.inner.liveness_client.get_sequencer_list().await;
    // }
}
