use std::sync::Arc;

use radius_sequencer_sdk::json_rpc::RpcClient;
use ssal::avs::LivenessClient;

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
    leader: RpcClient,
    followers: Vec<RpcClient>,

    liveness_client: Option<LivenessClient>,

    is_leader: bool,
}

pub struct Cluster {
    inner: Arc<ClusterInner>,
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
    pub fn new(
        rollup_id: RollupId,
        leader: RpcClient,
        followers: Vec<RpcClient>,
        liveness_client: Option<LivenessClient>,
    ) -> Self {
        let inner = ClusterInner {
            rollup_id,
            leader,
            followers,
            liveness_client,
            is_leader: false,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn rollup_id(&self) -> &String {
        &self.inner.rollup_id
    }

    pub fn leader(&self) -> &RpcClient {
        &self.inner.leader
    }

    pub fn followers(&self) -> &Vec<RpcClient> {
        &self.inner.followers
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

    pub fn get_liveness_client(&self) -> Option<&LivenessClient> {
        self.inner.liveness_client.as_ref()
    }
}
