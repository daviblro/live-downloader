use std::sync::{atomic::AtomicBool, Arc};

use crate::{monitoring::RecordingEngine, persistence::Database};

pub struct AppState {
    pub(crate) database: Arc<Database>,
    pub(crate) engine: RecordingEngine,
    pub(crate) is_quitting: AtomicBool,
}
