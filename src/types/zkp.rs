use serde::{Deserialize, Serialize};
use skde::delay_encryption::SkdeParams;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdeZkp;

#[derive(Debug, Clone)]
pub enum ZkpParams {
    Skde(SkdeParams),
}

impl ZkpParams {
    pub fn skde_params(&self) -> Option<&SkdeParams> {
        match self {
            ZkpParams::Skde(skde_params) => Some(skde_params),
        }
    }
}
