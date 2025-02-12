use std::{any::Any, sync::Arc};

use radius_sdk::{
    json_rpc::client::RpcClient,
    kvstore::{CachedKvStore, CachedKvStoreError},
    signature::PrivateKeySigner,
};
use skde::delay_encryption::SkdeParams;

use crate::{
    client::liveness_service_manager::{
        distributed_key_generation::DistributedKeyGenerationClient, seeder::SeederClient,
    },
    merkle_tree_manager::MerkleTreeManager,
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
    rpc_client: RpcClient,
    merkle_tree_manager: MerkleTreeManager,
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
        rpc_client: RpcClient,
        merkle_tree_manager: MerkleTreeManager,
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
            rpc_client,
            merkle_tree_manager,
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

    pub fn merkle_tree_manager(&self) -> &MerkleTreeManager {
        &self.inner.merkle_tree_manager
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
