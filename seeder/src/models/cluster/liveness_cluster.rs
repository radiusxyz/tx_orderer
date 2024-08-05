use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessClusterModel {
    pub platform: PlatForm,
    pub service_type: ServiceType,

    pub cluster_id: ClusterId,
    pub sequencer_addresses: Addresses,
}

impl LivenessClusterModel {
    pub fn new(platform: PlatForm, service_type: ServiceType, cluster_id: ClusterId) -> Self {
        Self {
            platform,
            cluster_id,
            service_type,

            sequencer_addresses: Addresses::new(),
        }
    }
}

impl LivenessClusterModel {
    pub const ID: &'static str = stringify!(LivenessClusterModel);

    pub fn get(
        platform: &PlatForm,
        service_type: &ServiceType,

        cluster_id: &ClusterId,
    ) -> Result<Self, DbError> {
        let key = (Self::ID, platform, service_type, cluster_id);
        database()?.get(&key)
    }

    pub fn get_mut(
        platform: &PlatForm,
        service_type: &ServiceType,

        cluster_id: &ClusterId,
    ) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, platform, service_type, cluster_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (
            Self::ID,
            &self.platform,
            &self.service_type,
            &self.cluster_id,
        );
        database()?.put(&key, self)
    }
}
