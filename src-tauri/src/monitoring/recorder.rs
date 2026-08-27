use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(super) struct ActiveJob {
    pub(super) target_id: String,
    pub(super) cancellation: CancellationToken,
}
