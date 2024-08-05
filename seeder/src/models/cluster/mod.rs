mod liveness_cluster;
mod validation_cluster;

pub use liveness_cluster::*;
use serde::{Deserialize, Serialize};
pub use validation_cluster::*;

use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterModel {
    Liveness(LivenessClusterModel),
    Validation(ValidationClusterModel),
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ClusterIdListModel {
    pub cluster_id_list: ClusterIdList,
}

impl ClusterIdListModel {
    pub fn new(cluster_id_list: ClusterIdList) -> Self {
        Self { cluster_id_list }
    }

    pub fn push(&mut self, cluster_id: ClusterId) {
        &self.cluster_id_list.push(cluster_id);
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
        database()?.get(&key)
    }

    pub fn entry(
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
    ) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, platform, sequencing_function_type, service_type);
        match database()?.get_mut(&key) {
            Ok(lock) => Ok(lock),
            Err(error) => {
                if error.is_none_type() {
                    let cluster_id_list_model = Self::new(ClusterIdList::default());

                    cluster_id_list_model.put(platform, sequencing_function_type, service_type)?;

                    Ok(database()?.get_mut(&key)?)
                } else {
                    Err(error)
                }
            }
        }
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
