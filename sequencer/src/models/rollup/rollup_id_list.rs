use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupIdListModel {
    rollup_id_list: RollupIdList,
}

impl RollupIdListModel {
    pub fn new(rollup_id_list: RollupIdList) -> Self {
        Self { rollup_id_list }
    }

    pub fn push(&mut self, rollup_id: RollupId) {
        &self.rollup_id_list.push(rollup_id);
    }

    pub fn rollup_id_list(&self) -> &RollupIdList {
        &self.rollup_id_list
    }

    pub fn add_rollup_id(&mut self, rollup_id: RollupId) {
        let is_exist_rollup_id = self.rollup_id_list.contains(&rollup_id);

        if !is_exist_rollup_id {
            self.rollup_id_list.push(rollup_id);
        }
    }
}

impl RollupIdListModel {
    pub const ID: &'static str = stringify!(RollupIdListModel);

    pub fn get(
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
        cluster_id: &ClusterId,
    ) -> Result<Self, DbError> {
        let key = (
            Self::ID,
            platform,
            sequencing_function_type,
            service_type,
            cluster_id,
        );
        database()?.get(&key)
    }

    pub fn entry(
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
        cluster_id: &ClusterId,
    ) -> Result<Lock<'static, Self>, DbError> {
        let key = (
            Self::ID,
            platform,
            sequencing_function_type,
            service_type,
            cluster_id,
        );
        match database()?.get_mut(&key) {
            Ok(lock) => Ok(lock),
            Err(error) => {
                if error.is_none_type() {
                    let rollup_id_list_model = Self::new(RollupIdList::default());

                    rollup_id_list_model.put(
                        platform,
                        sequencing_function_type,
                        service_type,
                        cluster_id,
                    )?;

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
        cluster_id: &ClusterId,
    ) -> Result<(), DbError> {
        let key = (
            Self::ID,
            &platform,
            &sequencing_function_type,
            &service_type,
            &cluster_id,
        );
        database()?.put(&key, self)
    }
}
