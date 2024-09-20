use std::{collections::BTreeMap, sync::Arc};

use radius_sequencer_sdk::context::SharedContext;

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

    liveness_clients: SharedContext<BTreeMap<(Platform, ServiceProvider), LivenessClient>>,

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
        liveness_clients: BTreeMap<(Platform, ServiceProvider), LivenessClient>,
        zkp_params: ZkpParams,
    ) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
            key_management_client: key_management_system_client,
            liveness_clients: SharedContext::from(liveness_clients),
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

    pub fn get_liveness_client(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Option<LivenessClient> {
        self.inner
            .liveness_clients
            .load()
            .as_ref()
            .get(&(platform, service_provider))
            .cloned()
    }

    pub fn key_management_system_client(&self) -> &KeyManagementSystemClient {
        &self.inner.key_management_client
    }

    pub fn zkp_params(&self) -> &ZkpParams {
        &self.inner.zkp_params
    }
}
