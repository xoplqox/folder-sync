use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionKind {
    SyncFile {
        rel_path: String,
        source_clone: String,
        target_clones: Vec<String>,
    },
    DeleteFile {
        rel_path: String,
        clones: Vec<String>,
    },
}

impl ActionKind {
    pub fn rel_path(&self) -> &str {
        match self {
            ActionKind::SyncFile { rel_path, .. } => rel_path,
            ActionKind::DeleteFile { rel_path, .. } => rel_path,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAction {
    pub id: Uuid,
    pub group_name: String,
    pub group_number: String,
    pub kind: ActionKind,
    pub status: ActionStatus,
    pub error: Option<String>,
    pub bytes_total: Option<u64>,
    pub bytes_done: u64,
    /// Groups file-level actions that were expanded from one folder-level
    /// request, so the UI can cluster and bulk-remove them together.
    pub group_id: Option<Uuid>,
}

impl BatchAction {
    pub fn new(group_name: String, group_number: String, kind: ActionKind, group_id: Option<Uuid>) -> Self {
        BatchAction {
            id: Uuid::new_v4(),
            group_name,
            group_number,
            kind,
            status: ActionStatus::Queued,
            error: None,
            bytes_total: None,
            bytes_done: 0,
            group_id,
        }
    }
}
