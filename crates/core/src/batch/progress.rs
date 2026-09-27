use serde::Serialize;
use uuid::Uuid;

use crate::batch::action::ActionStatus;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// Sent once when a client first connects, so it can sync to current
    /// state without waiting for the next real event.
    Snapshot,
    RunStarted,
    ActionStarted {
        action_id: Uuid,
    },
    ActionProgress {
        action_id: Uuid,
        bytes_done: u64,
        bytes_total: u64,
    },
    ActionFinished {
        action_id: Uuid,
        status: ActionStatus,
        error: Option<String>,
    },
    RunCancelled,
    RunCompleted {
        succeeded: usize,
        failed: usize,
    },
}

/// Destination for progress events during a batch run. Kept as a trait
/// (rather than a concrete `tokio::sync::broadcast::Sender`) so `core` stays
/// free of async-runtime dependencies; the server implements this on top of
/// a broadcast channel to fan events out to connected WebSocket clients.
pub trait ProgressSink: Send + Sync {
    fn emit(&self, event: ProgressEvent);
}

/// A sink that discards every event — useful for tests and any headless use
/// of the executor that doesn't care about live progress.
pub struct NullSink;

impl ProgressSink for NullSink {
    fn emit(&self, _event: ProgressEvent) {}
}
