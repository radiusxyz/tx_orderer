use std::sync::Arc;

use json_rpc::RpcClient;
use tokio::sync::Mutex;

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

    pub fn update(
        &mut self,
        my_address: Address,
        ssal_block_number: u64,
        rollup_block_number: u64,
    ) -> Result<u64, database::Error> {
        let previous_block_height = self.transaction_order;

        let sequencer_list = SequencerList::get(ssal_block_number)?;
        // # Safety
        // The length of the sequencer list must be constrained to be greater than 1 by the contract.
        let leader_index = rollup_block_number as usize % sequencer_list.len();
        let (leader_address, _) = sequencer_list.get_sequencer(leader_index).unwrap();

        self.ssal_block_number = ssal_block_number;
        self.rollup_block_number = rollup_block_number;
        self.transaction_order = 1;
        self.is_leader = my_address == *leader_address;

        Ok(previous_block_height)
    }
}

pub struct Cluster {
    inner: Arc<Mutex<ClusterInner>>,
}

struct ClusterInner {
    my_address: Address,
    leader_index: usize,
    sequencer_list: Vec<(Address, Option<RpcClient>)>,
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
    pub fn new(my_address: Address, leader_index: usize) -> Self {
        let inner = ClusterInner {
            my_address,
            leader_index,
            sequencer_list: Vec::with_capacity(30),
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn update(&self, sequencer_list: SequencerList) {
        let sequencer_list: Vec<(Address, Option<RpcClient>)> = sequencer_list
            .into_inner()
            .into_iter()
            .filter_map(|(address, rpc_url)| match rpc_url {
                Some(rpc_url) => {
                    let rpc_client = RpcClient::new(rpc_url, 1).ok();
                    Some((address, rpc_client))
                }
                None => Some((address, None)),
            })
            .collect();

        let mut inner_lock = self.inner.lock().await;
        inner_lock.sequencer_list = sequencer_list;
    }

    pub async fn rpc_client_list(&self) -> Vec<Option<RpcClient>> {
        let inner_lock = self.inner.lock().await;
        inner_lock
            .sequencer_list
            .iter()
            .filter(|(address, _)| *address != inner_lock.my_address)
            .map(|(_, rpc_client)| rpc_client.clone())
            .collect()
    }

    pub async fn leader_rpc_client(&self) -> Option<RpcClient> {
        let inner_lock = self.inner.lock().await;
        let (_, rpc_client) = inner_lock
            .sequencer_list
            .get(inner_lock.leader_index)
            .unwrap();

        rpc_client.clone()
    }
}
