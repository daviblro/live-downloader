use std::collections::{HashMap, HashSet, VecDeque};

use chrono::{DateTime, Utc};

use super::recorder::ActiveJob;

#[derive(Debug, Default)]
pub(super) struct EngineRuntime {
    pub(super) running: bool,
    pub(super) scheduler_started: bool,
    pub(super) next_tick_at: Option<DateTime<Utc>>,
    pub(super) active_jobs: HashMap<String, ActiveJob>,
    pub(super) probing_targets: HashSet<String>,
    pub(super) starting_targets: HashSet<String>,
    pub(super) queued_targets: VecDeque<String>,
    /// Targets whose recording the user stopped. They are not recorded again
    /// until a check sees the stream offline or the user checks them manually.
    pub(super) suppressed_targets: HashSet<String>,
}
