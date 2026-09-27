use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use folder_sync_core::config::ComparisonMode;
use folder_sync_core::drive::scan_drives;
use folder_sync_core::tree::{build_merged_tree, MergedTree};
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize, Default)]
pub struct TreeQuery {
    mode: Option<ComparisonMode>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/drives/:name/:number/tree", get(get_tree))
}

async fn get_tree(
    State(state): State<AppState>,
    Path((name, number)): Path<(String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<MergedTree>, ApiError> {
    let config = state.config();
    let mode = query.mode.unwrap_or(config.comparison_mode);

    let tree = tokio::task::spawn_blocking(move || -> Result<MergedTree, ApiError> {
        let groups = scan_drives(&config.scan_root)?;
        let group = groups
            .into_iter()
            .find(|g| g.key.name == name && g.key.number == number)
            .ok_or_else(|| ApiError::not_found(format!("no drive group {name}_{number} found")))?;

        let hasher = |path: &std::path::Path, size: u64, mtime: std::time::SystemTime| {
            state.0.hash_cache.get_or_hash(path, size, mtime).ok()
        };
        let hasher_ref: Option<&folder_sync_core::tree::HashFn> = match mode {
            ComparisonMode::NameSizeHash => Some(&hasher),
            ComparisonMode::NameSize => None,
        };

        Ok(build_merged_tree(&group, mode, hasher_ref)?)
    })
    .await
    .map_err(|e| ApiError {
        status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("tree build task failed: {e}"),
    })??;

    Ok(Json(tree))
}
