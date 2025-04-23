mod cluster_metadata;
use std::collections::{
    btree_set::{self, BTreeSet},
    BTreeMap,
};

pub use cluster_metadata::*;

use super::prelude::*;
use crate::{
    client::{
        liveness_service_manager::radius::{initialize_new_cluster, LivenessServiceManagerClient},
        seeder::TxOrdererRpcInfo,
    },
    error::Error,
    state::AppState,
};

pub type ClusterId = String;

#[derive(Default, Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, liveness_service_provider: LivenessServiceProvider, cluster_id: &str))]
pub struct LatestSyncedClusterBlockHeight(u64);

impl LatestSyncedClusterBlockHeight {
    pub fn get_block_height(&self) -> u64 {
        self.0
    }

    pub fn set_block_height(&mut self, block_height: u64) {
        self.0 = block_height;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, liveness_service_provider: LivenessServiceProvider))]
pub struct ClusterIdList(BTreeSet<ClusterId>);

impl ClusterIdList {
    pub fn insert(&mut self, cluster_id: impl AsRef<str>) {
        self.0.insert(cluster_id.as_ref().into());
    }

    pub fn remove(&mut self, cluster_id: impl AsRef<str>) {
        self.0.remove(cluster_id.as_ref());
    }

    pub fn iter(&self) -> btree_set::Iter<'_, ClusterId> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, liveness_service_provider: LivenessServiceProvider, cluster_id: &ClusterId, platform_block_height: u64))]
pub struct Cluster {
    #[serde(serialize_with = "serialize_address")]
    pub tx_orderer_address: Address,

    pub rollup_id_list: RollupIdList,
    pub tx_orderer_rpc_infos: BTreeMap<usize, TxOrdererRpcInfo>,

    pub block_margin: u64,
}

impl Cluster {
    pub fn new(
        tx_orderer_rpc_infos: BTreeMap<usize, TxOrdererRpcInfo>,
        rollup_id_list: RollupIdList,
        tx_orderer_address: Address,
        block_margin: u64,
    ) -> Self {
        Self {
            tx_orderer_rpc_infos,
            rollup_id_list,
            tx_orderer_address,
            block_margin,
        }
    }

    // pub async fn put_and_update_with_margin(
    //     cluster: &Cluster,
    //     platform: Platform,
    //     liveness_service_provider: LivenessServiceProvider,
    //     cluster_id: &ClusterId,
    //     platform_block_height: u64,
    // ) -> Result<(), KvStoreError> {
    //     Cluster::put(
    //         cluster,
    //         platform,
    //         liveness_service_provider,
    //         cluster_id,
    //         platform_block_height,
    //     )?;

    //     // Keep [`ClusterInfo`] for `Self::Margin` blocks.
    //     let block_height_for_remove =
    // platform_block_height.wrapping_sub(cluster.block_margin * 2);

    //     Cluster::delete(
    //         platform,
    //         liveness_service_provider,
    //         cluster_id,
    //         block_height_for_remove,
    //     )?;

    //     Ok(())
    // }

    pub fn get_tx_orderer_address_list(&self) -> Vec<Address> {
        self.tx_orderer_rpc_infos
            .values()
            .map(|tx_orderer_rpc_info| tx_orderer_rpc_info.tx_orderer_address.clone())
            .collect()
    }

    pub fn get_cluster_rpc_url_list(&self) -> Vec<String> {
        self.tx_orderer_rpc_infos
            .values()
            .filter_map(|tx_orderer_rpc_info| {
                if tx_orderer_rpc_info.cluster_rpc_url.is_none() {
                    return None;
                }

                Some(tx_orderer_rpc_info.cluster_rpc_url.to_owned().unwrap())
            })
            .collect()
    }

    pub fn get_other_cluster_rpc_url_list(&self) -> Vec<String> {
        self.tx_orderer_rpc_infos
            .values()
            .filter_map(|tx_orderer_rpc_info| {
                if tx_orderer_rpc_info.tx_orderer_address != self.tx_orderer_address {
                    if tx_orderer_rpc_info.cluster_rpc_url.is_none() {
                        return None;
                    }

                    Some(tx_orderer_rpc_info.cluster_rpc_url.to_owned().unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_others_external_rpc_url_list(&self) -> Vec<String> {
        self.tx_orderer_rpc_infos
            .values()
            .filter_map(|tx_orderer_rpc_info| {
                if tx_orderer_rpc_info.tx_orderer_address != self.tx_orderer_address {
                    if tx_orderer_rpc_info.external_rpc_url.is_none() {
                        return None;
                    }

                    Some(tx_orderer_rpc_info.external_rpc_url.to_owned().unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_tx_orderer_rpc_info(
        &self,
        tx_orderer_address: &Address,
    ) -> Option<TxOrdererRpcInfo> {
        self.tx_orderer_rpc_infos
            .iter()
            .find(|(_index, tx_orderer_rpc_info)| {
                tx_orderer_rpc_info.tx_orderer_address == tx_orderer_address
            })
            .map(|(_index, tx_orderer_rpc_info)| tx_orderer_rpc_info.clone())
    }

    pub fn register_tx_orderer(&mut self, index: usize, tx_orderer_rpc_info: TxOrdererRpcInfo) {
        self.tx_orderer_rpc_infos.insert(index, tx_orderer_rpc_info);
    }

    pub fn deregister_tx_orderer(&mut self, tx_orderer_address: &str) {
        let tx_orderer_index = self
            .tx_orderer_rpc_infos
            .iter()
            .find(|(_index, tx_orderer_rpc_info)| {
                tx_orderer_rpc_info.tx_orderer_address == tx_orderer_address
            })
            .map(|(index, _tx_orderer)| *index);

        if let Some(tx_orderer_index) = tx_orderer_index {
            self.tx_orderer_rpc_infos.remove(&tx_orderer_index);
        }
    }

    pub fn add_rollup(&mut self, rollup_id: &RollupId) {
        self.rollup_id_list.insert(rollup_id.to_owned());
    }
}

impl Cluster {
    pub async fn sync_cluster(
        context: AppState,
        cluster_id: &ClusterId,
        liveness_service_manager_client: &LivenessServiceManagerClient,
        platform_block_height: u64,
    ) -> Result<Cluster, Error> {
        let block_margin: u64 = liveness_service_manager_client
            .publisher()
            .get_block_margin()
            .await
            .expect("Failed to get block margin")
            .try_into()
            .expect("Failed to convert block margin");

        initialize_new_cluster(
            context,
            liveness_service_manager_client,
            cluster_id,
            platform_block_height,
            block_margin,
        )
        .await
        .unwrap();

        Cluster::get(
            liveness_service_manager_client.platform(),
            liveness_service_manager_client.service_provider(),
            cluster_id,
            platform_block_height,
        ).map_err(|e| {
            tracing::error!(
                "Failed to retrieve cluster - cluster_id: {:?} / platform_block_height: {:?} / error: {:?}",
                cluster_id,
                platform_block_height,
                e
            );

            Error::ClusterNotFound
        })
    }
}
