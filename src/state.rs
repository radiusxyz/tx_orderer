use std::{path::PathBuf, sync::Arc};

use radius_sequencer_sdk::context::SharedContext;

use crate::{client::liveness::seeder::SeederClient, types::*};

pub type AppState = Arc<State>;

pub struct State {
    signing_key_path: PathBuf,
    seeder: SeederClient,
}

impl State {
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
