use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupModel {
    rollup: Rollup,

    sequencing_info_key: SequencingInfoKey,
    cluster_id: ClusterId,
}

impl RollupModel {
    pub fn rollup(&self) -> &Rollup {
        &self.rollup
    }

    pub fn sequencing_info_key(&self) -> &SequencingInfoKey {
        &self.sequencing_info_key
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.cluster_id
    }
}

impl RollupModel {
    pub const ID: &'static str = stringify!(RollupModel);

    pub fn new(
        rollup: Rollup,
        sequencing_info_key: SequencingInfoKey,
        cluster_id: ClusterId,
    ) -> Self {
        Self {
            rollup,
            sequencing_info_key,
            cluster_id,
        }
    }

    pub fn get(rollup_id: &RollupId) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.rollup.rollup_id());
        database()?.put(&key, self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadataModel {
    cluster_id: ClusterId,
    rollup_id: RollupId,

    pub rollup_block_height: BlockHeight,
    pub liveness_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub is_leader: bool,
}

impl ClusterMetadataModel {
    pub fn new(cluster_id: ClusterId, rollup_id: RollupId) -> Self {
        Self {
            cluster_id,
            rollup_id,
            rollup_block_height: 0,
            liveness_block_height: 0,
            transaction_order: 0.into(),
            is_leader: false,
        }
    }

    pub fn get_transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn increment_transaction_order(&mut self) {
        self.transaction_order.increment();
    }
}

impl ClusterMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(cluster_id: &ClusterId) -> Result<Self, DbError> {
        let key = (Self::ID, cluster_id);
        database()?.get(&key)
    }

    pub fn get_mut(cluster_id: &ClusterId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, cluster_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.cluster_id.clone());
        database()?.put(&key, self)
    }
}

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
