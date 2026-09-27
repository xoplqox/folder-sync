use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use folder_sync_core::config::ComparisonMode;
use folder_sync_core::drive::scan_drives;
use folder_sync_core::tree::{build_merged_tree, MergedTree};

use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/drives/:name/:number/tree", get(get_tree))
}

async fn get_tree(
    State(state): State<AppState>,
    Path((name, number)): Path<(String, String)>,
) -> Result<Json<MergedTree>, ApiError> {
    let config = state.config();
    let groups = scan_drives(&config.scan_root)?;
    let group = groups
        .into_iter()
        .find(|g| g.key.name == name && g.key.number == number)
        .ok_or_else(|| ApiError::not_found(format!("no drive group {name}_{number} found")))?;

    // Advanced hash mode is wired up once the hashing milestone lands;
    // for now this always builds the name+size comparison.
    let tree = build_merged_tree(&group, ComparisonMode::NameSize, None)?;
    Ok(Json(tree))
}
