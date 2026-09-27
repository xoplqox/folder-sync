mod batch;
mod config;
mod drives;
mod health;
mod tree;

use axum::Router;

use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(config::router())
        .merge(drives::router())
        .merge(tree::router())
        .merge(batch::router())
}
