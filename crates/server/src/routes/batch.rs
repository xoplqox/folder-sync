use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use folder_sync_core::batch::{
    find_node, plan_delete_file, plan_delete_folder, plan_sync_file, plan_sync_folder, ActionKind, BatchAction, BatchRun,
};
use folder_sync_core::config::ComparisonMode;
use folder_sync_core::drive::scan_drives;
use folder_sync_core::tree::build_merged_tree;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/batch", get(get_batch))
        .route("/api/batch/actions", post(queue_action).delete(remove_by_group))
        .route("/api/batch/actions/:id", delete(remove_action))
}

async fn get_batch(State(state): State<AppState>) -> Json<BatchRun> {
    Json(state.0.batch_queue.snapshot())
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum QueueRequest {
    SyncFile {
        group_name: String,
        group_number: String,
        rel_path: String,
    },
    DeleteFile {
        group_name: String,
        group_number: String,
        rel_path: String,
    },
    SyncFolder {
        group_name: String,
        group_number: String,
        rel_path: String,
    },
    DeleteFolder {
        group_name: String,
        group_number: String,
        rel_path: String,
    },
}

impl QueueRequest {
    fn location(&self) -> (&str, &str, &str) {
        match self {
            QueueRequest::SyncFile { group_name, group_number, rel_path }
            | QueueRequest::DeleteFile { group_name, group_number, rel_path }
            | QueueRequest::SyncFolder { group_name, group_number, rel_path }
            | QueueRequest::DeleteFolder { group_name, group_number, rel_path } => (group_name, group_number, rel_path),
        }
    }
}

#[derive(Serialize)]
struct QueueResponseBody {
    queued: usize,
    skipped_conflicts: Vec<String>,
    run: BatchRun,
}

async fn queue_action(State(state): State<AppState>, Json(req): Json<QueueRequest>) -> Result<Json<QueueResponseBody>, ApiError> {
    let config = state.config();
    let (group_name, group_number, rel_path) = req.location();
    let (group_name, group_number, rel_path) = (group_name.to_string(), group_number.to_string(), rel_path.to_string());
    let mode = config.comparison_mode;
    let scan_root = config.scan_root.clone();
    let state_for_hash = state.clone();

    let (group_name_c, group_number_c) = (group_name.clone(), group_number.clone());
    let is_folder_request = matches!(req, QueueRequest::SyncFolder { .. } | QueueRequest::DeleteFolder { .. });

    let (actions, skipped_conflicts): (Vec<ActionKind>, Vec<String>) = tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
        let groups = scan_drives(&scan_root)?;
        let group = groups
            .into_iter()
            .find(|g| g.key.name == group_name_c && g.key.number == group_number_c)
            .ok_or_else(|| ApiError::not_found(format!("no drive group {group_name_c}_{group_number_c} found")))?;

        let hasher = |path: &std::path::Path, size: u64, mtime: std::time::SystemTime| {
            state_for_hash.0.hash_cache.get_or_hash(path, size, mtime).ok()
        };
        let hasher_ref: Option<&folder_sync_core::tree::HashFn> = match mode {
            ComparisonMode::NameSizeHash => Some(&hasher),
            ComparisonMode::NameSize => None,
        };
        let tree = build_merged_tree(&group, mode, hasher_ref)?;
        let node = find_node(&tree.root, &rel_path).ok_or_else(|| ApiError::not_found(format!("no such path: {rel_path}")))?;

        match req {
            QueueRequest::SyncFile { .. } => Ok((vec![plan_sync_file(node)?], Vec::new())),
            QueueRequest::DeleteFile { .. } => Ok((vec![plan_delete_file(node)?], Vec::new())),
            QueueRequest::SyncFolder { .. } => {
                let plan = plan_sync_folder(node);
                Ok((plan.actions, plan.skipped_conflicts))
            }
            QueueRequest::DeleteFolder { .. } => Ok((plan_delete_folder(node), Vec::new())),
        }
    })
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("batch planning task failed: {e}"),
    })??;

    let group_id = if is_folder_request && !actions.is_empty() {
        Some(Uuid::new_v4())
    } else {
        None
    };

    let queued = actions.len();
    let batch_actions: Vec<BatchAction> = actions
        .into_iter()
        .map(|kind| BatchAction::new(group_name.clone(), group_number.clone(), kind, group_id))
        .collect();
    state.0.batch_queue.add_many(batch_actions);

    Ok(Json(QueueResponseBody {
        queued,
        skipped_conflicts,
        run: state.0.batch_queue.snapshot(),
    }))
}

async fn remove_action(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<BatchRun>, ApiError> {
    state.0.batch_queue.remove(id)?;
    Ok(Json(state.0.batch_queue.snapshot()))
}

#[derive(Deserialize)]
struct GroupQuery {
    group_id: Uuid,
}

async fn remove_by_group(State(state): State<AppState>, Query(query): Query<GroupQuery>) -> Json<BatchRun> {
    state.0.batch_queue.remove_group(query.group_id);
    Json(state.0.batch_queue.snapshot())
}
