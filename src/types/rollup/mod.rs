use crate::types::prelude::*;

mod model;
mod rollup;
mod rollup_metadata;

pub use model::*;
pub use rollup::*;
pub use rollup_metadata::*;

pub type RollupIdList = Vec<String>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupType {
    PolygonCdk,
}
