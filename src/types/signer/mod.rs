mod model;

use std::collections::btree_set::{BTreeSet, Iter};

pub use model::*;

use super::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SignerList(BTreeSet<Platform>);

impl SignerList {
    pub fn insert(&mut self, platform: Platform) {
        self.0.insert(platform);
    }

    pub fn remove(&mut self, platform: Platform) {
        self.0.remove(&platform);
    }

    pub fn iter(&self) -> Iter<'_, Platform> {
        self.0.iter()
    }
}
