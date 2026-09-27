pub mod action;
pub mod plan;

pub use action::{ActionKind, ActionStatus, BatchAction};
pub use plan::{find_node, plan_delete_file, plan_delete_folder, plan_resolve_conflict, plan_sync_file, plan_sync_folder, FolderPlan, PlanError};

use std::sync::RwLock;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Idle,
    Running,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchRun {
    pub actions: Vec<BatchAction>,
    pub status: RunStatus,
}

impl Default for BatchRun {
    fn default() -> Self {
        BatchRun {
            actions: Vec::new(),
            status: RunStatus::Idle,
        }
    }
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("no queued action with that id")]
    NotFound,
    #[error("cannot remove an action that is currently running")]
    Running,
}

/// The server-side, in-memory-only batch queue. Lives for the process's
/// lifetime (no disk persistence), so a browser reload is fine but a
/// process restart discards it — matches the app's "resume while the app
/// stays open" requirement.
pub struct BatchQueue {
    run: RwLock<BatchRun>,
}

impl Default for BatchQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchQueue {
    pub fn new() -> Self {
        BatchQueue {
            run: RwLock::new(BatchRun::default()),
        }
    }

    pub fn snapshot(&self) -> BatchRun {
        self.run.read().expect("batch queue lock poisoned").clone()
    }

    pub fn add(&self, action: BatchAction) {
        self.run.write().expect("batch queue lock poisoned").actions.push(action);
    }

    pub fn add_many(&self, actions: Vec<BatchAction>) {
        self.run.write().expect("batch queue lock poisoned").actions.extend(actions);
    }

    pub fn remove(&self, id: Uuid) -> Result<(), RemoveError> {
        let mut run = self.run.write().expect("batch queue lock poisoned");
        let pos = run.actions.iter().position(|a| a.id == id).ok_or(RemoveError::NotFound)?;
        if run.actions[pos].status == ActionStatus::Running {
            return Err(RemoveError::Running);
        }
        run.actions.remove(pos);
        Ok(())
    }

    pub fn remove_group(&self, group_id: Uuid) {
        let mut run = self.run.write().expect("batch queue lock poisoned");
        run.actions.retain(|a| a.group_id != Some(group_id) || a.status == ActionStatus::Running);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::action::ActionKind;

    fn sync_action() -> BatchAction {
        BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "f.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into()],
            },
            None,
        )
    }

    #[test]
    fn add_list_remove_round_trip() {
        let queue = BatchQueue::new();
        let action = sync_action();
        let id = action.id;
        queue.add(action);
        assert_eq!(queue.snapshot().actions.len(), 1);

        queue.remove(id).unwrap();
        assert_eq!(queue.snapshot().actions.len(), 0);
    }

    #[test]
    fn remove_missing_action_errors() {
        let queue = BatchQueue::new();
        assert!(matches!(queue.remove(Uuid::new_v4()), Err(RemoveError::NotFound)));
    }

    #[test]
    fn remove_group_clears_all_matching_actions() {
        let queue = BatchQueue::new();
        let group_id = Uuid::new_v4();
        let mut a = sync_action();
        a.group_id = Some(group_id);
        let mut b = sync_action();
        b.group_id = Some(group_id);
        let unrelated = sync_action();

        queue.add(a);
        queue.add(b);
        queue.add(unrelated);
        assert_eq!(queue.snapshot().actions.len(), 3);

        queue.remove_group(group_id);
        assert_eq!(queue.snapshot().actions.len(), 1);
    }
}
