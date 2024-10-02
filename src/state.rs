use std::{any::Any, sync::Arc};

use radius_sequencer_sdk::{
    kvstore::{CachedKvStore, CachedKvStoreError},
    signature::PrivateKeySigner,
};
use skde::SkdeParams;

use crate::{
    client::liveness::{key_management_system::KeyManagementSystemClient, seeder::SeederClient},
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
    validation_clients: CachedKvStore,
    signers: CachedKvStore,

    pvde_params: PvdeParams,
    skde_params: SkdeParams,
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
        signers: CachedKvStore,
        liveness_clients: CachedKvStore,
        validation_clients: CachedKvStore,
        pvde_params: PvdeParams,
        skde_params: SkdeParams,
    ) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
            key_management_client: key_management_system_client,
            signers,
            liveness_clients,
            validation_clients,
            pvde_params,
            skde_params,
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

    pub async fn add_validation_client<T>(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        validation_client: T,
    ) -> Result<(), CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, service_provider);

        self.inner
            .validation_clients
            .put(key, validation_client)
            .await
    }

    pub async fn get_validation_client<T>(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Result<T, CachedKvStoreError>
    where
        T: Clone + Any + Send + 'static,
    {
        let key = &(platform, service_provider);

        self.inner.validation_clients.get(key).await
    }

    pub fn key_management_system_client(&self) -> &KeyManagementSystemClient {
        &self.inner.key_management_client
    }

    pub fn pvde_params(&self) -> &PvdeParams {
        &self.inner.pvde_params
    }

    pub fn skde_params(&self) -> &SkdeParams {
        &self.inner.skde_params
    }
}
