use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupModel {
    rollup: Rollup,
}

impl RollupModel {
    pub fn rollup(&self) -> &Rollup {
        &self.rollup
    }
}

impl RollupModel {
    pub const ID: &'static str = stringify!(RollupModel);

    pub fn new(rollup: Rollup) -> Self {
        Self { rollup }
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
