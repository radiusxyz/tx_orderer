use std::sync::Arc;

use json_rpc::RpcClient;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    pub ssal_block_number: u64,
    pub rollup_block_number: u64,
    pub transaction_order: u64,
    pub is_leader: bool,
}

impl Default for ClusterMetadata {
    fn default() -> Self {
        Self {
            ssal_block_number: 0,
            rollup_block_number: 0,
            transaction_order: 0,
            is_leader: false,
        }
    }
}

impl ClusterMetadata {
    const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, database::Error> {
        database()?.get_mut(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }

    pub fn new(
        ssal_block_number: u64,
        rollup_block_number: u64,
        transaction_order: u64,
        is_leader: bool,
    ) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            transaction_order,
            is_leader,
        }
    }

    pub fn issue_order_commitment(
        &mut self,
        user_transaction: &UserTransaction,
    ) -> Result<OrderCommitment, database::Error> {
        self.transaction_order += 1;
        user_transaction.put(self.rollup_block_number, self.transaction_order)?;

        Ok(OrderCommitment::new(
            self.rollup_block_number,
            self.transaction_order,
        ))
    }

    pub async fn update(
        &mut self,
        my_address: Address,
        cluster_id: &String,
        ssal_block_number: u64,
        rollup_block_number: u64,
    ) -> Result<Cluster, Error> {
        let mut sequencer_list = SequencerList::get(ssal_block_number)?.into_inner();

        // # Safety
        // The length of the sequencer list must be constrained to be greater than 1 by the contract.
        let leader_index = rollup_block_number
            .checked_rem(sequencer_list.len() as u64)
            .ok_or(Error::EmptySequencerList)? as usize;
        if leader_index >= sequencer_list.len() {
            return Err(Error::LeaderIndexOutofBound);
        }

        // Separate the leader from followers.
        let (leader_address, leader_rpc_url) = sequencer_list.remove(leader_index);
        let leader_rpc_url = leader_rpc_url.ok_or(Error::EmptyLeaderRpcUrl)?;

        // Build the leader RPC client.
        let leader = RpcClient::new(leader_rpc_url)?;

        // Build the followers' RPC client.
        let followers: Vec<RpcClient> = sequencer_list
            .into_iter()
            .filter(|(address, rpc_url)| *address != my_address && rpc_url.is_some())
            .filter_map(|(_, rpc_url)| RpcClient::new(rpc_url.unwrap()).ok())
            .collect();

        // Update the cluster.

        self.ssal_block_number = ssal_block_number;
        self.rollup_block_number = rollup_block_number;
        self.transaction_order = 1;
        self.is_leader = my_address == leader_address;

        Ok(Cluster::new(cluster_id.to_owned(), leader, followers))
    }
}

pub struct Cluster {
    inner: Arc<ClusterInner>,
}

struct ClusterInner {
    id: String,
    leader: RpcClient,
    followers: Vec<RpcClient>,
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
}
