use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use folder_sync_core::config::Config;

#[derive(Clone)]
pub struct AppState(pub Arc<AppStateInner>);

pub struct AppStateInner {
    pub read_only: bool,
    pub config_path: PathBuf,
    pub config: RwLock<Config>,
}

impl AppState {
    pub fn new(read_only: bool, config_path: PathBuf, config: Config) -> Self {
        Self(Arc::new(AppStateInner {
            read_only,
            config_path,
            config: RwLock::new(config),
        }))
    }

    pub fn config(&self) -> Config {
        self.0.config.read().expect("config lock poisoned").clone()
    }
}
