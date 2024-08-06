use std::sync::Arc;

use crate::{state::AppState, types::prelude::*};

pub struct SyncInfo {
    sequencing_info: SequencingInfo,
    app_state: Arc<AppState>,
}

impl SyncInfo {
    pub fn new(sequencing_info: SequencingInfo, app_state: Arc<AppState>) -> Self {
        Self {
            sequencing_info,
            app_state,
        }
    }

    pub fn sequencing_info(&self) -> &SequencingInfo {
        &self.sequencing_info
    }

    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }
}
