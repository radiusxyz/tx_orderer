use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupIdListModel {
    rollup_id_list: RollupIdList,
}

impl RollupIdListModel {
    pub fn rollup_id_list(&self) -> &RollupIdList {
        &self.rollup_id_list
    }

    pub fn rollup_id_list_mut(&mut self) -> &mut RollupIdList {
        &mut self.rollup_id_list
    }
}

impl RollupIdListModel {
    pub const ID: &'static str = stringify!(RollupIdListModel);

    pub fn new(rollup_id_list: RollupIdList) -> Self {
        Self { rollup_id_list }
    }

    pub fn get() -> Result<Self, DbError> {
        match database()?.get(&Self::ID) {
            Ok(model) => Ok(model),
            Err(error) => {
                if error.is_none_type() {
                    let rollup_id_list_model = Self::new(RollupIdList::default());

                    rollup_id_list_model.put()?;

                    Ok(rollup_id_list_model)
                } else {
                    Err(error)
                }
            }
        }
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, DbError> {
        database()?.get_mut(&Self::ID)
    }

    pub fn put(&self) -> Result<(), DbError> {
        database()?.put(&Self::ID, self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadataModel {
    rollup_id: RollupId,
    pub rollup_block_height: BlockHeight,
    pub liveness_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub is_leader: bool,
}

impl ClusterMetadataModel {
    pub fn new(
        rollup_id: RollupId,
        liveness_block_height: BlockHeight,
        rollup_block_height: BlockHeight,
        transaction_order: TransactionOrder,
        is_leader: bool,
    ) -> Self {
        Self {
            rollup_id,
            liveness_block_height,
            rollup_block_height,
            transaction_order,
            is_leader,
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

    pub fn get(rollup_id: &RollupId) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.rollup_id.clone());
        database()?.put(&key, self)
    }
}
