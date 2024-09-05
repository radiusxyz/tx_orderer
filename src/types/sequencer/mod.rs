use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SequencerStatus {
    Uninitialized,
    Running,
}
