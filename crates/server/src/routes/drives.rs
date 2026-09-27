use std::path::PathBuf;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use folder_sync_core::drive::scan_drives;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct DriveGroupSummary {
    pub name: String,
    pub number: String,
    pub clone_count: usize,
    pub clones: Vec<String>,
}

#[derive(Serialize)]
pub struct DrivesResponse {
    pub scan_root: PathBuf,
    pub scan_root_exists: bool,
    pub groups: Vec<DriveGroupSummary>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/drives", get(list_drives))
        .route("/api/drives/rescan", post(list_drives))
}

async fn list_drives(State(state): State<AppState>) -> Json<DrivesResponse> {
    let config = state.config();
    let scan_root = config.scan_root;
    let scan_root_exists = scan_root.is_dir();

    let groups = if scan_root_exists {
        scan_drives(&scan_root).unwrap_or_default()
    } else {
        Vec::new()
    };

    let groups = groups
        .into_iter()
        .map(|g| DriveGroupSummary {
            name: g.key.name,
            number: g.key.number,
            clone_count: g.clones.len(),
            clones: g.clones.into_iter().map(|d| d.label.clone).collect(),
        })
        .collect();

    Json(DrivesResponse {
        scan_root,
        scan_root_exists,
        groups,
    })
}
