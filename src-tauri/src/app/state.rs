use std::sync::{atomic::AtomicBool, Arc};

use crate::{monitoring::RecordingEngine, persistence::Database, twitch::TwitchService};

pub struct AppState {
    pub(crate) database: Arc<Database>,
    pub(crate) engine: RecordingEngine,
    pub(crate) twitch: Arc<TwitchService>,
    pub(crate) is_quitting: AtomicBool,
}
