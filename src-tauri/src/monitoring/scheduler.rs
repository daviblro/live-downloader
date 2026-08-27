use std::collections::{HashMap, HashSet, VecDeque};

use super::recorder::ActiveJob;

#[derive(Debug, Default)]
pub(super) struct EngineRuntime {
    pub(super) running: bool,
    pub(super) scheduler_started: bool,
    pub(super) active_jobs: HashMap<String, ActiveJob>,
    pub(super) probing_targets: HashSet<String>,
    pub(super) starting_targets: HashSet<String>,
    pub(super) queued_targets: VecDeque<String>,
}
