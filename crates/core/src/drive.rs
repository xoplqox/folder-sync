use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

use crate::error::CoreError;

/// Matches partition labels of the form `{Name}_{Number}{Clone}`,
/// e.g. `Daten_1a`, `Videos_2ab`. Name is alphanumeric, Number is
/// digits, Clone is one or more lowercase letters (so clone counts
/// beyond 26 can use `aa`, `ab`, ... like spreadsheet columns).
fn label_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([A-Za-z0-9]+)_(\d+)([a-z]+)$").expect("valid regex"))
}

/// Compares clone letter sequences the way spreadsheet columns sort:
/// shorter sequences first, then lexicographically ("a" < "b" < ... < "z" < "aa" < "ab").
fn clone_order_key(clone: &str) -> (usize, &str) {
    (clone.len(), clone)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DriveLabel {
    pub name: String,
    pub number: String,
    pub clone: String,
    pub raw: String,
}

impl DriveLabel {
    /// Parses a partition/directory label like `Daten_1a` into its parts.
    /// Returns `None` if the label doesn't match the expected pattern.
    pub fn parse(raw: &str) -> Option<Self> {
        let caps = label_regex().captures(raw)?;
        Some(DriveLabel {
            name: caps[1].to_string(),
            number: caps[2].to_string(),
            clone: caps[3].to_string(),
            raw: raw.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct DriveGroupKey {
    pub name: String,
    pub number: String,
}

impl DriveGroupKey {
    pub fn display(&self) -> String {
        format!("{}_{}", self.name, self.number)
    }
}

impl PartialOrd for DriveGroupKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DriveGroupKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase()).then_with(|| {
            // Numeric comparison when both parse cleanly, so "2" sorts before "10";
            // falls back to string comparison for non-numeric/odd inputs.
            match (self.number.parse::<u64>(), other.number.parse::<u64>()) {
                (Ok(a), Ok(b)) => a.cmp(&b).then_with(|| self.number.cmp(&other.number)),
                _ => self.number.cmp(&other.number),
            }
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Drive {
    pub label: DriveLabel,
    pub mount_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveGroup {
    pub key: DriveGroupKey,
    pub clones: Vec<Drive>,
}

/// Scans `root`'s immediate subdirectories for names matching the
/// `{Name}_{Number}{Clone}` pattern and groups them by `{Name}_{Number}`.
/// Non-matching entries (and non-directories) are silently skipped.
pub fn scan_drives(root: &Path) -> Result<Vec<DriveGroup>, CoreError> {
    if !root.is_dir() {
        return Err(CoreError::InvalidScanRoot(root.to_path_buf()));
    }

    let mut groups: Vec<DriveGroup> = Vec::new();

    let entries = std::fs::read_dir(root).map_err(|source| CoreError::Io {
        path: root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(label) = DriveLabel::parse(name) else {
            continue;
        };

        let key = DriveGroupKey {
            name: label.name.clone(),
            number: label.number.clone(),
        };
        let drive = Drive {
            label,
            mount_path: path,
        };

        match groups.iter_mut().find(|g| g.key == key) {
            Some(group) => group.clones.push(drive),
            None => groups.push(DriveGroup {
                key,
                clones: vec![drive],
            }),
        }
    }

    for group in &mut groups {
        group.clones.sort_by(|a, b| clone_order_key(&a.label.clone).cmp(&clone_order_key(&b.label.clone)));
    }
    groups.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_valid_labels() {
        let label = DriveLabel::parse("Daten_1a").unwrap();
        assert_eq!(label.name, "Daten");
        assert_eq!(label.number, "1");
        assert_eq!(label.clone, "a");
    }

    #[test]
    fn parses_multi_letter_clone() {
        let label = DriveLabel::parse("Videos_2ab").unwrap();
        assert_eq!(label.clone, "ab");
    }

    #[test]
    fn rejects_non_matching_names() {
        assert!(DriveLabel::parse("Daten1a").is_none()); // missing underscore
        assert!(DriveLabel::parse("Daten_a").is_none()); // missing number
        assert!(DriveLabel::parse("Daten_1A").is_none()); // uppercase clone
        assert!(DriveLabel::parse("_1a").is_none()); // missing name
        assert!(DriveLabel::parse("Daten_1").is_none()); // missing clone
    }

    #[test]
    fn groups_by_name_and_number_and_sorts_clones() {
        let dir = tempdir().unwrap();
        for name in ["Videos_2c", "Videos_2a", "Daten_1b", "Daten_1a", "Daten_10a", "not_matching", "Daten_2a"] {
            std::fs::create_dir(dir.path().join(name)).unwrap();
        }
        // a stray file should be ignored, not treated as a drive
        std::fs::write(dir.path().join("Daten_1x_stray.txt"), b"x").unwrap();

        let groups = scan_drives(dir.path()).unwrap();

        let keys: Vec<String> = groups.iter().map(|g| g.key.display()).collect();
        // alphabetical by name, then numeric by number: Daten_1 < Daten_2 < Daten_10 < Videos_2
        assert_eq!(keys, vec!["Daten_1", "Daten_2", "Daten_10", "Videos_2"]);

        let daten1 = &groups[0];
        let clone_letters: Vec<&str> = daten1.clones.iter().map(|d| d.label.clone.as_str()).collect();
        assert_eq!(clone_letters, vec!["a", "b"]);
    }

    #[test]
    fn clone_letters_sort_like_spreadsheet_columns() {
        let dir = tempdir().unwrap();
        for name in ["Daten_1z", "Daten_1aa", "Daten_1a", "Daten_1ab"] {
            std::fs::create_dir(dir.path().join(name)).unwrap();
        }
        let groups = scan_drives(dir.path()).unwrap();
        let clone_letters: Vec<&str> = groups[0].clones.iter().map(|d| d.label.clone.as_str()).collect();
        assert_eq!(clone_letters, vec!["a", "z", "aa", "ab"]);
    }

    #[test]
    fn missing_root_is_an_error() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        assert!(scan_drives(&missing).is_err());
    }
}
