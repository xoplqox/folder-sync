use std::path::PathBuf;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use folder_sync_core::config::{save_config, ComparisonMode};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct ConfigResponse {
    pub scan_root: PathBuf,
    pub comparison_mode: ComparisonMode,
    pub read_only: bool,
}

#[derive(Deserialize, Default)]
pub struct ConfigUpdate {
    pub scan_root: Option<PathBuf>,
    pub comparison_mode: Option<ComparisonMode>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/config", get(get_config).put(update_config))
}

fn to_response(state: &AppState, config: &folder_sync_core::config::Config) -> ConfigResponse {
    ConfigResponse {
        scan_root: config.scan_root.clone(),
        comparison_mode: config.comparison_mode,
        read_only: state.0.read_only,
    }
}

async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let config = state.config();
    Json(to_response(&state, &config))
}

async fn update_config(
    State(state): State<AppState>,
    Json(update): Json<ConfigUpdate>,
) -> Result<Json<ConfigResponse>, ApiError> {
    let config = {
        let mut guard = state.0.config.write().expect("config lock poisoned");
        if let Some(root) = update.scan_root {
            guard.scan_root = root;
        }
        if let Some(mode) = update.comparison_mode {
            guard.comparison_mode = mode;
        }
        save_config(&state.0.config_path, &guard)?;
        guard.clone()
    };
    Ok(Json(to_response(&state, &config)))
}
