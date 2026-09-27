mod config;
mod drives;
mod health;

use axum::Router;

use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(config::router())
        .merge(drives::router())
}
