use std::sync::Arc;

use haphazard::{AtomicPtr, Domain, HazardPointer};
use ssal::ethereum::SsalClient;

pub struct State {
    inner: Arc<AtomicPtr<StateInner>>,
}

struct StateInner {
    ssal_client: SsalClient,
}

unsafe impl Send for State {}

unsafe impl Sync for State {}
