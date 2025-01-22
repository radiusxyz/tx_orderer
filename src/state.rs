use std::{any::Any, sync::Arc};

use radius_sdk::{
    json_rpc::client::RpcClient,
    kvstore::{CachedKvStore, CachedKvStoreError, Value},
    signature::PrivateKeySigner,
};
use skde::delay_encryption::SkdeParams;

use crate::{
    client::liveness::{
        distributed_key_generation::DistributedKeyGenerationClient, seeder::SeederClient,
    },
    error::Error,
    profiler::Profiler,
    types::*,
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    seeder_client: SeederClient,
    distributed_key_generation_client: DistributedKeyGenerationClient,

    liveness_clients: CachedKvStore,
    validation_clients: CachedKvStore,
    signers: CachedKvStore,

    skde_params: SkdeParams,

    profiler: Option<Profiler>,

    clusters: CachedKvStore,
    rollups: CachedKvStore,
    rollup_metadatas: CachedKvStore,

    rpc_client: RpcClient,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        seeder_client: SeederClient,
        distributed_key_generation_client: DistributedKeyGenerationClient,
        signers: CachedKvStore,
        liveness_clients: CachedKvStore,
        validation_clients: CachedKvStore,
        skde_params: SkdeParams,
        profiler: Option<Profiler>,
        rollup_metadatas: CachedKvStore,
        rollups: CachedKvStore,
        clusters: CachedKvStore,
        rpc_client: RpcClient,
    ) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
            distributed_key_generation_client,
            signers,
            liveness_clients,
            validation_clients,
            skde_params,
            profiler,
            rollup_metadatas,
            rollups,
            clusters,
            rpc_client,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn seeder_client(&self) -> &SeederClient {
        &self.inner.seeder_client
    }

    pub fn distributed_key_generation_client(&self) -> &DistributedKeyGenerationClient {
        &self.inner.distributed_key_generation_client
    }

    pub fn skde_params(&self) -> &SkdeParams {
        &self.inner.skde_params
    }

    pub fn profiler(&self) -> Option<Profiler> {
        self.inner.profiler.clone()
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.inner.rpc_client
    }
}

/// Rollup metadata functions
impl AppState {
    pub async fn add_rollup_metadata(
        &self,
        rollup_id: &str,
        rollup_metadata: RollupMetadata,
    ) -> Result<(), Error> {
        self.inner
            .rollup_metadatas
            .put(&rollup_id, rollup_metadata)
            .await
            .map_err(Error::CachedKvStore)?;

        Ok(())
    }

    pub async fn get_rollup_metadata(&self, rollup_id: &str) -> Result<RollupMetadata, Error> {
        match self.inner.rollup_metadatas.get(&rollup_id).await {
            Ok(rollup_metadata) => Ok(rollup_metadata),
            Err(_) => {
                tracing::warn!(
                    "RollupMetadata not found in cache - rollup_id: {}",
                    rollup_id
                );

                let rollup_metadata = RollupMetadata::get(rollup_id).map_err(Error::Database)?;

                self.add_rollup_metadata(rollup_id, rollup_metadata.clone())
                    .await?;

                Ok(rollup_metadata)
            }
        }
    }

    pub async fn get_mut_rollup_metadata(
        &self,
        rollup_id: &str,
    ) -> Result<Value<RollupMetadata>, Error> {
        match self.inner.rollup_metadatas.get_mut(&rollup_id).await {
            Ok(rollup_metadata) => Ok(rollup_metadata),
            Err(_) => {
                let rollup_metadata = RollupMetadata::get(rollup_id).map_err(Error::Database)?;

                self.add_rollup_metadata(rollup_id, rollup_metadata.clone())
                    .await?;

                self.inner
                    .rollup_metadatas
                    .get_mut(&rollup_id)
                    .await
                    .map_err(Error::CachedKvStore)
            }
        }
    }
}

/// Cluster functions
impl AppState {
    pub async fn add_cluster(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        platform_block_height: u64,
        cluster: Cluster,
    ) -> Result<(), Error> {
        let key = &(
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );

        self.inner
            .clusters
            .put(key, cluster)
            .await
            .map_err(Error::CachedKvStore)?;

        Ok(())
    }

    pub async fn delete_cluster(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        platform_block_height: u64,
    ) -> Result<(), Error> {
        let key = &(
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );

        self.inner
            .clusters
            .delete::<(Platform, ServiceProvider, &str, u64), Cluster>(key)
            .await
            .map_err(Error::CachedKvStore)?;

        Ok(())
    }

    pub async fn get_cluster(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        platform_block_height: u64,
    ) -> Result<Cluster, Error> {
        let key = &(
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );

        match self.inner.clusters.get(key).await {
            Ok(cluster) => Ok(cluster),
            Err(_) => {
                tracing::warn!(
                    "Cluster not found in cache - platform block_height: {}",
                    platform_block_height
                );

                let cluster = Cluster::get(
                    platform,
                    service_provider,
                    cluster_id,
                    platform_block_height,
                )
                .map_err(Error::Database)?;

                self.add_cluster(
                    platform,
                    service_provider,
                    cluster_id,
                    platform_block_height,
                    cluster.clone(),
                )
                .await?;

                Ok(cluster)
            }
        }
    }

    pub async fn get_mut_cluster(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        platform_block_height: u64,
    ) -> Result<Value<Cluster>, Error> {
        let key = &(
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        );

        match self.inner.clusters.get_mut(key).await {
            Ok(rollup_metadata) => Ok(rollup_metadata),
            Err(_) => {
                let cluster = Cluster::get(
                    platform,
                    service_provider,
                    cluster_id,
                    platform_block_height,
                )
                .map_err(Error::Database)?;

                self.add_cluster(
                    platform,
                    service_provider,
                    cluster_id,
                    platform_block_height,
                    cluster.clone(),
                )
                .await?;

                self.inner
                    .clusters
                    .get_mut(key)
                    .await
                    .map_err(Error::CachedKvStore)
            }
        }
    }
}

/// Rollup functions
impl AppState {
    pub async fn add_rollup(&self, rollup_id: &str, rollup: Rollup) -> Result<(), Error> {
        let key = &(rollup_id);

        self.inner
            .rollups
            .put(key, rollup)
            .await
            .map_err(Error::CachedKvStore)?;

        Ok(())
    }

    pub async fn get_rollup(&self, rollup_id: &str) -> Result<Rollup, Error> {
        let key = &(rollup_id);

        match self.inner.rollups.get(key).await {
            Ok(rollup) => Ok(rollup),
            Err(_) => {
                tracing::warn!("Rollup not found in cache - rollup_id: {}", rollup_id);

                let rollup = Rollup::get(rollup_id).map_err(Error::Database)?;

                self.add_rollup(rollup_id, rollup.clone()).await?;

                Ok(rollup)
            }
        }
    }

    pub async fn get_mut_rollup(&self, rollup_id: &str) -> Result<Value<Rollup>, Error> {
        match self.inner.rollups.get_mut(&rollup_id).await {
            Ok(rollup) => Ok(rollup),
            Err(_) => {
                let rollup = Rollup::get(rollup_id).map_err(Error::Database)?;

                self.add_rollup(rollup_id, rollup.clone()).await?;

                self.inner
                    .rollups
                    .get_mut(&rollup_id)
                    .await
                    .map_err(Error::CachedKvStore)
            }
        }
    }
}

/// Validation client functions
impl AppState {
    pub async fn add_validation_client<T>(
        &self,
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
        validation_client: T,
    ) -> Result<(), CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, validation_service_provider);

        self.inner
            .validation_clients
            .put(key, validation_client)
            .await
    }

    pub async fn get_validation_client<T>(
        &self,
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
    ) -> Result<T, CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, validation_service_provider);

        self.inner.validation_clients.get(key).await
    }
}

/// Liveness client functions
impl AppState {
    pub async fn add_liveness_client<T>(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        liveness_client: T,
    ) -> Result<(), CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, service_provider);

        self.inner.liveness_clients.put(key, liveness_client).await
    }

    pub async fn get_liveness_client<T>(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<T, CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, service_provider);

        self.inner.liveness_clients.get(key).await
    }
}

/// Signer functions
impl AppState {
    pub async fn add_signer(
        &self,
        platform: Platform,
        signer: PrivateKeySigner,
    ) -> Result<(), CachedKvStoreError> {
        let key = &(platform);

        self.inner.signers.put(key, signer).await
    }

    pub async fn get_signer(
        &self,
        platform: Platform,
    ) -> Result<PrivateKeySigner, CachedKvStoreError> {
        let key = &(platform);

        self.inner.signers.get(key).await
    }
}
