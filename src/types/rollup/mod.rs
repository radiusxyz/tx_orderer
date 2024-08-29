use crate::types::prelude::*;

mod rollup;
mod rollup_metadata;

pub use rollup::*;
pub use rollup_metadata::*;

pub type RollupId = String;
pub type RollupIdList = Vec<RollupId>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupType {
    PolygonCdk,
}
