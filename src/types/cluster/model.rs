use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterIdListModel;

impl ClusterIdListModel {
    const ID: &'static str = stringify!(ClusterIdListModel);

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id_list: &ClusterIdList,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.put(key, cluster_id_list)
    }

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

    pub fn delete(
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, platform, service_provider);

        kvstore()?.delete(key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterModel;

impl ClusterModel {
    const ID: &'static str = stringify!(ClusterInfoModel);

    pub fn put(
        cluster_id: &String,
        platform_block_height: u64,
        cluster: &Cluster,
    ) -> Result<(), KvStoreError> {
        let database = kvstore()?;

        let put_key = &(Self::ID, cluster_id, platform_block_height);
        database.put(put_key, cluster)?;

        // Keep [`ClusterInfo`] for `Self::Margin` blocks.
        let delete_key = &(
            Self::ID,
            platform_block_height.wrapping_sub(cluster.block_margin()),
            cluster_id,
        );
        database.delete(delete_key)?;

        Ok(())
    }

    pub fn get(cluster_id: &String, platform_block_height: u64) -> Result<Cluster, KvStoreError> {
        let key = &(Self::ID, cluster_id, platform_block_height);

        kvstore()?.get(key)
    }
}
