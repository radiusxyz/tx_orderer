use std::{collections::HashMap, path::PathBuf};

use crate::{client::liveness::seeder::SeederClient, types::*};

pub struct AppState {
    signing_key_path: PathBuf,
    seeder: SeederClient,
}

impl AppState {
    pub fn signing_key_path(&self) -> &PathBuf {
        &self.signing_key_path
    }

    pub fn seeder(&self) -> &SeederClient {
        &self.seeder
    }

    pub fn is_using_zkp(&self) -> bool {
        true
    }
}
