use super::prelude::*;

pub const CURRENT_CODE_VERSION: &str = "v0.1.0";
pub const REQURIED_DATABASE_VERSION: &str = "v0.0.2";

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key())]
pub struct Version {
    pub code_version: String,
    pub database_version: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            code_version: CURRENT_CODE_VERSION.to_string(),
            database_version: "v0.0.1".to_string(),
        }
    }
}
