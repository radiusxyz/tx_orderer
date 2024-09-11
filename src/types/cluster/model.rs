use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterIdListModel;

impl ClusterIdListModel {
    const ID: &'static str = stringify!(ClusterIdListModel);

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<ClusterIdList, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get(key)
    }

    pub fn get_or_default(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<ClusterIdList, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get_or_default(key)
    }

    pub fn get_mut(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<Lock<'static, ClusterIdList>, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get_mut(key)
    }

    pub fn get_mut_or_default(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<Lock<'static, ClusterIdList>, KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.get_mut_or_default(key)
    }

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id_list: &ClusterIdList,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.put(key, cluster_id_list)
    }

    pub fn delete(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.delete(key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterInfoModel;

impl ClusterInfoModel {
    const ID: &'static str = stringify!(ClusterInfoModel);

    pub fn get(
        liveness_block_number: u64,
        cluster_id: &String,
    ) -> Result<ClusterInfo, KvStoreError> {
        let key = &(Self::ID, liveness_block_number, cluster_id);

        kvstore()?.get(key)
    }

    pub fn put(
        cluster_id: &String,
        liveness_block_number: u64,
        cluster: &ClusterInfo,
    ) -> Result<(), KvStoreError> {
        let database = kvstore()?;

        let put_key = &(Self::ID, liveness_block_number, cluster_id);
        database.put(put_key, cluster)?;

        // Keep [`ClusterInfo`] for `Self::Margin` blocks.
        let delete_key = &(
            Self::ID,
            liveness_block_number.wrapping_sub(cluster.block_margin()),
            cluster_id,
        );
        database.delete(delete_key)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadataModel;

impl ClusterMetadataModel {
    const ID: &'static str = stringify!(ClusterMetadataModel);

    pub fn get(
        rollup_id: &String,
        rollup_block_height: u64,
    ) -> Result<ClusterMetadata, KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.get(key)
    }

    pub fn put(
        rollup_id: &String,
        rollup_block_height: u64,
        cluster_metadata: &ClusterMetadata,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.put(key, cluster_metadata)
    }
}
