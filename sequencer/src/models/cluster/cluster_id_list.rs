use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ClusterIdListModel {
    pub cluster_id_list: ClusterIdList,
}

impl ClusterIdListModel {
    pub fn new(cluster_id_list: ClusterIdList) -> Self {
        Self { cluster_id_list }
    }

    pub fn add_cluster_id(&mut self, cluster_id: ClusterId) {
        let is_exist_cluster_id = self.cluster_id_list.contains(&cluster_id);

        if !is_exist_cluster_id {
            self.cluster_id_list.push(cluster_id);
        }
    }
}

impl ClusterIdListModel {
    pub const ID: &'static str = stringify!(ClusterIdListModel);

    pub fn get(
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
    ) -> Result<Self, DbError> {
        let key = (Self::ID, platform, sequencing_function_type, service_type);
        match database()?.get(&key) {
            Ok(cluster_id_list_model) => Ok(cluster_id_list_model),
            Err(error) => {
                if error.is_none_type() {
                    Ok(Self::new(ClusterIdList::default()))
                } else {
                    Err(error)
                }
            }
        }
    }

    pub fn get_mut(
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
    ) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, platform, sequencing_function_type, service_type);

        database()?.get_mut(&key)
    }

    pub fn put(
        &self,
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
    ) -> Result<(), DbError> {
        let key = (
            Self::ID,
            &platform,
            &sequencing_function_type,
            &service_type,
        );
        database()?.put(&key, self)
    }
}
