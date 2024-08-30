use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessClusterModel {
    pub platform: Platform,
    pub service_type: ServiceType,

    pub cluster_id: ClusterId,

    // TODO: change this property to RpcClients
    pub sequencer_address_list: AddressList,
}

impl LivenessClusterModel {
    pub fn new(platform: Platform, service_type: ServiceType, cluster_id: ClusterId) -> Self {
        Self {
            platform,
            cluster_id,
            service_type,

            sequencer_address_list: AddressList::new(),
        }
    }

    pub fn set_sequencer_list(&mut self, sequencer_addresses: AddressList) {
        self.sequencer_address_list = sequencer_addresses;
    }

    pub fn add_seqeuncer(&mut self, sequencer_address: Address) {
        let is_exist_sequencer_address = self.sequencer_address_list.contains(&sequencer_address);

        if !is_exist_sequencer_address {
            self.sequencer_address_list.push(sequencer_address);
        }
    }

    pub fn remove_sequencer(&mut self, sequencer_address: &Address) {
        let sequencer_address_list = self
            .sequencer_address_list
            .iter()
            .filter(|&address| address != sequencer_address)
            .cloned()
            .collect();

        self.set_sequencer_list(sequencer_address_list);
    }
}

impl LivenessClusterModel {
    pub const ID: &'static str = stringify!(LivenessClusterModel);

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
