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
pub struct ClusterBlockHeightModel;

impl ClusterBlockHeightModel {
    const ID: &'static str = stringify!(ClusterBlockHeightModel);

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        platform_block_height: u64,
    ) -> Result<(), KvStoreError> {
        let database = kvstore()?;

        let key = &(Self::ID, platform, service_provider, cluster_id);
        database.put(key, &platform_block_height)?;

        Ok(())
    }

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
    ) -> Result<u64, KvStoreError> {
        let key = &(Self::ID, platform, service_provider, cluster_id);

        kvstore()?.get(key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterModel;

impl ClusterModel {
    const ID: &'static str = stringify!(ClusterInfoModel);

    pub fn put(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        platform_block_height: u64,
        cluster: &Cluster,
    ) -> Result<(), KvStoreError> {
        let database = kvstore()?;

        let key = &(
            Self::ID,
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );
        database.put(key, cluster)?;

        // Keep [`ClusterInfo`] for `Self::Margin` blocks.
        let block_height_for_remove = platform_block_height.wrapping_sub(cluster.block_margin());

        ClusterBlockHeightModel::put(
            platform,
            service_provider,
            cluster_id,
            block_height_for_remove + 1,
        )?;

        let key = &(
            Self::ID,
            platform,
            service_provider,
            cluster_id,
            block_height_for_remove,
        );

        database.delete(key)?;

        Ok(())
    }

    pub fn get(
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        platform_block_height: u64,
    ) -> Result<Cluster, KvStoreError> {
        let key = &(
            Self::ID,
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );

        kvstore()?.get(key)
    }
}
