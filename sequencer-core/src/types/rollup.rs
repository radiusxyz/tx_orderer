use std::collections::HashSet;

use crate::{
    caller,
    error::Error,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RollupId(String);

impl AsRef<str> for RollupId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for RollupId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for RollupId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RollupSet(HashSet<RollupId>);

impl RollupSet {
    pub fn register(&mut self, rollup_id: RollupId) -> Result<(), Error> {
        match self.0.insert(rollup_id) {
            true => Ok(()),
            false => Err(Error::str_error(
                caller!(RollupSet::register()),
                "Rollup already exists.",
            )),
        }
    }

    pub fn contains(&self, rollup_id: &RollupId) -> bool {
        self.0.contains(rollup_id)
    }

    pub fn deregister(&mut self, rollup_id: &RollupId) -> Result<(), Error> {
        match self.0.remove(rollup_id) {
            true => Ok(()),
            false => Err(Error::str_error(
                caller!(RollupSet::deregister()),
                "Rollup already removed.",
            )),
        }
    }
}
