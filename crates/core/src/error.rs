use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("config error: {0}")]
    Config(String),
    #[error("scan root does not exist or is not a directory: {0}")]
    InvalidScanRoot(PathBuf),
}
