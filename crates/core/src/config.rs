use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonMode {
    /// Compare by filename + file size only (default).
    NameSize,
    /// Additionally compare file content via a fast (XXH3) hash.
    NameSizeHash,
}

impl Default for ComparisonMode {
    fn default() -> Self {
        ComparisonMode::NameSize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scan_root: PathBuf,
    #[serde(default)]
    pub comparison_mode: ComparisonMode,
}

impl Config {
    /// Default scan root: `/media/$USER`.
    pub fn default_scan_root() -> PathBuf {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());
        PathBuf::from("/media").join(user)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scan_root: Config::default_scan_root(),
            comparison_mode: ComparisonMode::default(),
        }
    }
}

/// Returns the default, XDG-compliant path to the config file
/// (`~/.config/folder-sync/config.toml` on Linux).
pub fn config_file_path() -> Result<PathBuf, CoreError> {
    let proj_dirs = ProjectDirs::from("", "", "folder-sync")
        .ok_or_else(|| CoreError::Config("could not determine a config directory for this user".into()))?;
    Ok(proj_dirs.config_dir().join("config.toml"))
}

/// Loads the config file at `path`, or returns built-in defaults if it doesn't exist yet.
pub fn load_config(path: &Path) -> Result<Config, CoreError> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(path).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&content).map_err(|e| CoreError::Config(format!("invalid config file {}: {e}", path.display())))
}

/// Persists `config` to `path` as TOML, creating parent directories as needed.
pub fn save_config(path: &Path, config: &Config) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CoreError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let content = toml::to_string_pretty(config).map_err(|e| CoreError::Config(e.to_string()))?;
    std::fs::write(path, content).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Loads the persisted config at `config_path`, applies an optional CLI
/// `--scan-root` override on top, and persists the result back to disk so
/// the override is remembered on the next launch (even without the flag).
pub fn resolve_config(config_path: &Path, cli_scan_root: Option<PathBuf>) -> Result<Config, CoreError> {
    let mut config = load_config(config_path)?;
    let mut dirty = !config_path.exists();

    if let Some(root) = cli_scan_root {
        config.scan_root = root;
        dirty = true;
    }

    if dirty {
        save_config(config_path, &config)?;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = load_config(&path).unwrap();
        assert_eq!(config.comparison_mode, ComparisonMode::NameSize);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = Config {
            scan_root: PathBuf::from("/media/alice"),
            comparison_mode: ComparisonMode::NameSizeHash,
        };
        save_config(&path, &config).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.scan_root, PathBuf::from("/media/alice"));
        assert_eq!(loaded.comparison_mode, ComparisonMode::NameSizeHash);
    }

    #[test]
    fn cli_scan_root_overrides_and_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");

        let config = resolve_config(&path, Some(PathBuf::from("/media/bob"))).unwrap();
        assert_eq!(config.scan_root, PathBuf::from("/media/bob"));
        assert!(path.exists());

        // A later launch without the flag should remember the override.
        let config_again = resolve_config(&path, None).unwrap();
        assert_eq!(config_again.scan_root, PathBuf::from("/media/bob"));
    }

    #[test]
    fn first_run_without_cli_flag_persists_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(!path.exists());
        resolve_config(&path, None).unwrap();
        assert!(path.exists());
    }
}
