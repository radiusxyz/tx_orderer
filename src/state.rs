use std::sync::Arc;

use radius_sequencer_sdk::{
    kvstore::{CachedKvStore, CachedKvStoreError},
    signature::PrivateKeySigner,
};

use crate::{
    client::liveness::{
        key_management_system::KeyManagementSystemClient, radius::LivenessClient,
        seeder::SeederClient,
    },
    types::*,
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}
struct AppStateInner {
    config: Config,
    seeder_client: SeederClient,
    key_management_client: KeyManagementSystemClient,

    liveness_clients: CachedKvStore,
    signers: CachedKvStore,

    zkp_params: ZkpParams,
}

unsafe impl Send for AppState {}

unsafe impl Sync for AppState {}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl AppState {
    pub fn new(
        config: Config,
        seeder_client: SeederClient,
        key_management_system_client: KeyManagementSystemClient,
        liveness_clients: CachedKvStore,
        signers: CachedKvStore,
        zkp_params: ZkpParams,
    ) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
            key_management_client: key_management_system_client,
            liveness_clients,
            signers,
            zkp_params,
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

    pub async fn get_liveness_client(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<LivenessClient, CachedKvStoreError> {
        let key = &(platform, service_provider);

        self.inner.liveness_clients.get(key).await
    }

    pub async fn add_liveness_client(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        liveness_client: LivenessClient,
    ) -> Result<(), CachedKvStoreError> {
        let key = &(platform, service_provider);

        self.inner.liveness_clients.put(key, liveness_client).await
    }

    pub async fn get_signer(
        &self,
        platform: Platform,
    ) -> Result<PrivateKeySigner, CachedKvStoreError> {
        let key = &(platform);

        self.inner.signers.get(key).await
    }

    pub async fn add_signer(
        &self,
        platform: Platform,
        signer: PrivateKeySigner,
    ) -> Result<(), CachedKvStoreError> {
        let key = &(platform);

        self.inner.signers.put(key, signer).await
    }

    pub fn key_management_system_client(&self) -> &KeyManagementSystemClient {
        &self.inner.key_management_client
    }

    pub fn zkp_params(&self) -> &ZkpParams {
        &self.inner.zkp_params
    }
}
