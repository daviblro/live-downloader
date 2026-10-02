use std::path::PathBuf;

use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(super) struct ActiveJob {
    pub(super) target_id: String,
    pub(super) pid: u32,
    pub(super) cancellation: CancellationToken,
    pub(super) output: PartialOutput,
}

/// Where a recording writes its file, used to salvage partial output when the
/// recorder is stopped before yt-dlp can finalise it.
#[derive(Debug, Clone)]
pub(super) struct PartialOutput {
    pub(super) directory: PathBuf,
    pub(super) timestamp: String,
}
