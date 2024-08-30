use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationClusterModel {
    pub platform: Platform,
    pub service_type: ServiceType,

    pub cluster_id: ClusterId,
    pub validator_address_list: AddressList,
}

impl ValidationClusterModel {
    pub fn new(platform: Platform, service_type: ServiceType, cluster_id: ClusterId) -> Self {
        Self {
            platform,
            service_type,

            cluster_id,
            validator_address_list: AddressList::new(),
        }
    }

    pub fn set_validator_list(&mut self, validator_addresses: AddressList) {
        self.validator_address_list = validator_addresses;
    }

    pub fn add_seqeuncer(&mut self, validator_address: Address) {
        let is_exist_validator_address = self.validator_address_list.contains(&validator_address);

        if !is_exist_validator_address {
            self.validator_address_list.push(validator_address);
        }
    }

    pub fn remove_validator(&mut self, validator_address: &Address) {
        let validator_address_list = self
            .validator_address_list
            .iter()
            .filter(|&address| address != validator_address)
            .cloned()
            .collect();

        self.set_validator_list(validator_address_list);
    }
}

impl ValidationClusterModel {
    pub const ID: &'static str = stringify!(ValidationClusterModel);

    pub fn get(
        platform: &Platform,
        service_type: &ServiceType,

        cluster_id: &ClusterId,
    ) -> Result<Self, DbError> {
        let key = (Self::ID, platform, service_type, cluster_id);
        database()?.get(&key)
    }

    pub fn get_mut(
        platform: &Platform,
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
