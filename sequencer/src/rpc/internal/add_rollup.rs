use crate::{
    models::{RollupIdListModel, RollupModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddRollup {
    rollup_id: RollupId,
}

impl AddRollup {
    pub const METHOD_NAME: &'static str = stringify!(AddRollup);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_id_list_model = RollupIdListModel::get_mut()?;

        rollup_id_list_model
            .rollup_id_list_mut()
            .push(parameter.rollup_id.clone());
        rollup_id_list_model.update()?;

        let rollup = Rollup::new(parameter.rollup_id);

        let rollup_model = RollupModel::new(rollup);
        rollup_model.put()?;

        Ok(())
    }
}
