use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use folder_sync_core::batch::{BatchExecutor, BatchQueue, ProgressEvent};
use folder_sync_core::config::Config;
use folder_sync_core::hash::HashCache;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState(pub Arc<AppStateInner>);

pub struct AppStateInner {
    pub read_only: bool,
    pub config_path: PathBuf,
    pub config: RwLock<Config>,
    pub hash_cache: HashCache,
    pub batch_queue: Arc<BatchQueue>,
    pub batch_executor: Arc<BatchExecutor>,
    pub progress_tx: broadcast::Sender<ProgressEvent>,
}

impl AppState {
    pub fn new(read_only: bool, config_path: PathBuf, config: Config) -> Self {
        let batch_queue = Arc::new(BatchQueue::new());
        let batch_executor = Arc::new(BatchExecutor::new(batch_queue.clone()));
        let (progress_tx, _rx) = broadcast::channel(256);
        Self(Arc::new(AppStateInner {
            read_only,
            config_path,
            config: RwLock::new(config),
            hash_cache: HashCache::new(),
            batch_queue,
            batch_executor,
            progress_tx,
        }))
    }

    pub fn config(&self) -> Config {
        self.0.config.read().expect("config lock poisoned").clone()
    }
}
