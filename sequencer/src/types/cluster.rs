use std::sync::Arc;

use json_rpc::RpcClient;

struct ClusterInner {
    id: String,
    leader: RpcClient,
    followers: Vec<RpcClient>,
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
    pub fn new(cluster_id: String, leader: RpcClient, followers: Vec<RpcClient>) -> Self {
        let inner = ClusterInner {
            id: cluster_id,
            leader,
            followers,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn id(&self) -> &String {
        &self.inner.id
    }

    pub fn leader(&self) -> &RpcClient {
        &self.inner.leader
    }

    pub fn followers(&self) -> &Vec<RpcClient> {
        &self.inner.followers
    }

    // TODO:
    // pub async fn update(
    //     &mut self,
    //     my_address: Address,
    //     cluster_id: &String,
    //     ssal_block_height: BlockHeight,
    //     rollup_block_height: BlockHeight,
    // ) -> Result<Cluster, Error> {
    //     let mut sequencer_list = SequencerList::get(ssal_block_height)?.into_inner();

    //     // # Safety
    //     // The length of the sequencer list must be constrained to be greater than 1 by the contract.
    //     let leader_index = rollup_block_height
    //         .checked_rem(sequencer_list.len() as u64)
    //         .ok_or(Error::EmptySequencerList)? as usize;
    //     if leader_index >= sequencer_list.len() {
    //         return Err(Error::LeaderIndexOutofBound);
    //     }

    //     // Separate the leader from followers.
    //     let (leader_address, leader_rpc_url) = sequencer_list.remove(leader_index);
    //     let leader_rpc_url = leader_rpc_url.ok_or(Error::EmptyLeaderRpcUrl)?;

    //     // Build the leader RPC client.
    //     let leader = RpcClient::new(leader_rpc_url)?;

    //     // Build the followers' RPC client.
    //     let followers: Vec<RpcClient> = sequencer_list
    //         .into_iter()
    //         .filter(|(address, rpc_url)| *address != my_address && rpc_url.is_some())
    //         .filter_map(|(_, rpc_url)| RpcClient::new(rpc_url.unwrap()).ok())
    //         .collect();

    //     self.ssal_block_height = ssal_block_height;
    //     self.rollup_block_height = rollup_block_height;
    //     self.transaction_order = 0;
    //     self.is_leader = my_address == leader_address;

    //     Ok(Cluster::new(cluster_id.to_owned(), leader, followers))
    // }
}
