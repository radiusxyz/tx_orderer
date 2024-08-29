use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupIdListModel {
    rollup_id_list: RollupIdList,
}

impl RollupIdListModel {
    pub fn new(rollup_id_list: RollupIdList) -> Self {
        Self { rollup_id_list }
    }

    pub fn rollup_id_list(self) -> RollupIdList {
        self.rollup_id_list
    }

    pub fn is_exist_rollup_id(&self, rollup_id: &RollupId) -> bool {
        self.rollup_id_list.contains(rollup_id)
    }

    pub fn add_rollup_id(&mut self, rollup_id: RollupId) {
        let is_exist_rollup_id = self.rollup_id_list.contains(&rollup_id);

        if !is_exist_rollup_id {
            self.rollup_id_list.push(rollup_id);
        }
    }

    pub fn update_rollup_id_list(&mut self, rollup_id_list: RollupIdList) {
        self.rollup_id_list = rollup_id_list;
    }
}

impl RollupIdListModel {
    pub const ID: &'static str = stringify!(RollupIdListModel);

    pub fn get() -> Result<Self, DbError> {
        let key = Self::ID;
        database()?.get(&key)
    }

    // change func name or separate
    pub fn get_mut() -> Result<Lock<'static, Self>, DbError> {
        let key = Self::ID;
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = Self::ID;
        database()?.put(&key, self)
    }
}
