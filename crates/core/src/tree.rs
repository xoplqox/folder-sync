use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

use serde::Serialize;
use walkdir::WalkDir;

use crate::compare::{compute_reference, match_state, FileMeta};
use crate::config::ComparisonMode;
use crate::drive::DriveGroup;
use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchState {
    /// Exists in this clone and matches the group's reference value for the active mode.
    Present,
    /// Exists in this clone but differs (size, or content hash in advanced mode).
    Differs,
    /// Does not exist in this clone.
    Missing,
}

/// Aggregate state of a folder's subtree for one clone, worst-state-wins
/// over all descendant files (Missing > Differs > Present).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupState {
    Identical,
    PartiallyDiffers,
    PartiallyMissing,
    /// The subtree contains no files at all (in any clone).
    Empty,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloneStatus {
    pub clone: String,
    pub state: MatchState,
    pub size: Option<u64>,
    pub mtime_unix: Option<i64>,
    pub hash: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RollupCloneStatus {
    pub clone: String,
    pub state: RollupState,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergedNode {
    pub name: String,
    pub kind: EntryKind,
    /// Forward-slash-separated path relative to the group root; stable identifier for API calls.
    pub rel_path: String,
    /// Per-clone status; populated for files, empty for folders (see `rollup`).
    pub clones: Vec<CloneStatus>,
    pub children: Vec<MergedNode>,
    /// Per-clone rollup status; populated for folders, `None` for files.
    pub rollup: Option<Vec<RollupCloneStatus>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergedTree {
    pub group_name: String,
    pub group_number: String,
    pub comparison_mode: ComparisonMode,
    pub root: MergedNode,
}

/// Optional hook for computing a file's content hash, used only when
/// `ComparisonMode::NameSizeHash` is active. Wired to a real (cached, XXH3)
/// implementation starting in the advanced-mode milestone; `None` here
/// simply leaves `hash` unset, so advanced mode without a hasher degrades
/// gracefully to a size-only comparison.
pub type HashFn<'a> = dyn Fn(&Path, u64, SystemTime) -> Option<u64> + Sync + 'a;

enum BuildChild {
    File(BuildFileNode),
    Folder(BuildFolderNode),
}

struct BuildFileNode {
    display_name: String,
    per_clone: BTreeMap<String, FileMeta>,
}

struct BuildFolderNode {
    display_name: String,
    children: BTreeMap<String, BuildChild>,
}

impl BuildFolderNode {
    fn new(display_name: String) -> Self {
        BuildFolderNode {
            display_name,
            children: BTreeMap::new(),
        }
    }

    /// Descends into (creating as needed) the folder child at `component`, preserving the
    /// first-seen display casing across clones.
    fn folder_child(&mut self, component: &str) -> &mut BuildFolderNode {
        let key = component.to_lowercase();
        let entry = self
            .children
            .entry(key)
            .or_insert_with(|| BuildChild::Folder(BuildFolderNode::new(component.to_string())));
        match entry {
            BuildChild::Folder(f) => f,
            BuildChild::File(_) => {
                // A path was a file in one clone and a folder in another (unusual, but
                // fault-tolerant handling: prefer treating it as a folder from here on).
                *entry = BuildChild::Folder(BuildFolderNode::new(component.to_string()));
                match entry {
                    BuildChild::Folder(f) => f,
                    BuildChild::File(_) => unreachable!(),
                }
            }
        }
    }

    fn file_child(&mut self, component: &str) -> &mut BuildFileNode {
        let key = component.to_lowercase();
        let entry = self
            .children
            .entry(key)
            .or_insert_with(|| BuildChild::File(BuildFileNode {
                display_name: component.to_string(),
                per_clone: BTreeMap::new(),
            }));
        match entry {
            BuildChild::File(f) => f,
            BuildChild::Folder(_) => {
                *entry = BuildChild::File(BuildFileNode {
                    display_name: component.to_string(),
                    per_clone: BTreeMap::new(),
                });
                match entry {
                    BuildChild::File(f) => f,
                    BuildChild::Folder(_) => unreachable!(),
                }
            }
        }
    }
}

fn to_unix(t: Option<SystemTime>) -> Option<i64> {
    t.and_then(|st| st.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
}

/// Builds a merged directory tree across all clones in `group`, comparing files per `mode`.
pub fn build_merged_tree(group: &DriveGroup, mode: ComparisonMode, hasher: Option<&HashFn>) -> Result<MergedTree, CoreError> {
    let clone_letters: Vec<String> = group.clones.iter().map(|d| d.label.clone.clone()).collect();
    let mut root = BuildFolderNode::new(String::new());

    for drive in &group.clones {
        let clone = &drive.label.clone;
        for entry in WalkDir::new(&drive.mount_path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            let rel = match entry.path().strip_prefix(&drive.mount_path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let components: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
            let Some((last, parents)) = components.split_last() else {
                continue;
            };

            let mut folder = &mut root;
            for parent in parents {
                folder = folder.folder_child(parent);
            }

            let file_type = entry.file_type();
            if file_type.is_dir() {
                folder.folder_child(last);
            } else if file_type.is_file() {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let size = metadata.len();
                let mtime = metadata.modified().ok();
                let hash = match (mode, hasher, mtime) {
                    (ComparisonMode::NameSizeHash, Some(f), Some(mt)) => f(entry.path(), size, mt),
                    _ => None,
                };
                let file_node = folder.file_child(last);
                file_node.per_clone.insert(
                    clone.clone(),
                    FileMeta {
                        size,
                        mtime_unix: to_unix(mtime),
                        hash,
                    },
                );
            }
            // Symlinks and other special file types are skipped: exFAT has no
            // symlink support, so real backup drives won't produce them.
        }
    }

    let root_node = convert_folder("", &root, &clone_letters, mode);

    Ok(MergedTree {
        group_name: group.key.name.clone(),
        group_number: group.key.number.clone(),
        comparison_mode: mode,
        root: root_node,
    })
}

fn convert_folder(rel_path: &str, folder: &BuildFolderNode, clone_letters: &[String], mode: ComparisonMode) -> MergedNode {
    let mut children: Vec<MergedNode> = folder
        .children
        .values()
        .map(|child| match child {
            BuildChild::File(f) => convert_file(rel_path, f, clone_letters, mode),
            BuildChild::Folder(sub) => {
                let child_rel = join_rel(rel_path, &sub.display_name);
                convert_folder(&child_rel, sub, clone_letters, mode)
            }
        })
        .collect();

    children.sort_by(|a, b| match (a.kind, b.kind) {
        (EntryKind::Folder, EntryKind::File) => std::cmp::Ordering::Less,
        (EntryKind::File, EntryKind::Folder) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    let rollup = clone_letters
        .iter()
        .map(|clone| RollupCloneStatus {
            clone: clone.clone(),
            state: rollup_for_clone(clone, &children),
        })
        .collect();

    MergedNode {
        name: folder.display_name.clone(),
        kind: EntryKind::Folder,
        rel_path: rel_path.to_string(),
        clones: Vec::new(),
        children,
        rollup: Some(rollup),
    }
}

fn rollup_for_clone(clone: &str, children: &[MergedNode]) -> RollupState {
    let mut saw_any_file = false;
    let mut has_missing = false;
    let mut has_differs = false;

    fn visit(clone: &str, node: &MergedNode, saw_any_file: &mut bool, has_missing: &mut bool, has_differs: &mut bool) {
        match node.kind {
            EntryKind::File => {
                *saw_any_file = true;
                if let Some(status) = node.clones.iter().find(|c| c.clone == clone) {
                    match status.state {
                        MatchState::Missing => *has_missing = true,
                        MatchState::Differs => *has_differs = true,
                        MatchState::Present => {}
                    }
                }
            }
            EntryKind::Folder => {
                for child in &node.children {
                    visit(clone, child, saw_any_file, has_missing, has_differs);
                }
            }
        }
    }

    for child in children {
        visit(clone, child, &mut saw_any_file, &mut has_missing, &mut has_differs);
    }

    if !saw_any_file {
        RollupState::Empty
    } else if has_missing {
        RollupState::PartiallyMissing
    } else if has_differs {
        RollupState::PartiallyDiffers
    } else {
        RollupState::Identical
    }
}

fn convert_file(parent_rel: &str, file: &BuildFileNode, clone_letters: &[String], mode: ComparisonMode) -> MergedNode {
    let present: Vec<(&str, &FileMeta)> = clone_letters
        .iter()
        .filter_map(|c| file.per_clone.get(c).map(|m| (c.as_str(), m)))
        .collect();
    let reference = compute_reference(mode, &present);

    let clones = clone_letters
        .iter()
        .map(|clone| {
            let meta = file.per_clone.get(clone);
            CloneStatus {
                clone: clone.clone(),
                state: match_state(mode, meta, reference),
                size: meta.map(|m| m.size),
                mtime_unix: meta.and_then(|m| m.mtime_unix),
                hash: meta.and_then(|m| m.hash),
            }
        })
        .collect();

    MergedNode {
        name: file.display_name.clone(),
        kind: EntryKind::File,
        rel_path: join_rel(parent_rel, &file.display_name),
        clones,
        children: Vec::new(),
        rollup: None,
    }
}

fn join_rel(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drive::{scan_drives};
    use tempfile::tempdir;

    fn write(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn basic_mode_flags_missing_and_size_diff_but_not_content_diff() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/identical.txt"), b"same");
        write(&dir.path().join("Daten_1b/identical.txt"), b"same");

        write(&dir.path().join("Daten_1a/only_a.txt"), b"only in a");

        write(&dir.path().join("Daten_1a/same_size.txt"), b"AAAA");
        write(&dir.path().join("Daten_1b/same_size.txt"), b"BBBB"); // same size, diff content

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();

        let find = |name: &str| tree.root.children.iter().find(|n| n.name == name).unwrap();

        let identical = find("identical.txt");
        assert!(identical.clones.iter().all(|c| c.state == MatchState::Present));

        let only_a = find("only_a.txt");
        let b_status = only_a.clones.iter().find(|c| c.clone == "b").unwrap();
        assert_eq!(b_status.state, MatchState::Missing);

        let same_size = find("same_size.txt");
        // Basic mode can't see content differences when size matches.
        assert!(same_size.clones.iter().all(|c| c.state == MatchState::Present));
    }

    #[test]
    fn size_difference_is_caught_in_basic_mode() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"AAAA");
        write(&dir.path().join("Daten_1b/f.txt"), b"AAAAAAAA");

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();

        let f = tree.root.children.iter().find(|n| n.name == "f.txt").unwrap();
        let b_status = f.clones.iter().find(|c| c.clone == "b").unwrap();
        assert_eq!(b_status.state, MatchState::Differs);
    }

    #[test]
    fn nested_folder_rollup_reflects_missing_descendant() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/docs/notes/note1.txt"), b"x");
        write(&dir.path().join("Daten_1b/docs/notes/note1.txt"), b"x");
        write(&dir.path().join("Daten_1a/docs/notes/note2.txt"), b"y");
        // note2.txt missing from clone b

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();

        let docs = tree.root.children.iter().find(|n| n.name == "docs").unwrap();
        let rollup = docs.rollup.as_ref().unwrap();
        let a_rollup = rollup.iter().find(|r| r.clone == "a").unwrap();
        let b_rollup = rollup.iter().find(|r| r.clone == "b").unwrap();
        assert_eq!(a_rollup.state, RollupState::Identical);
        assert_eq!(b_rollup.state, RollupState::PartiallyMissing);
    }

    #[test]
    fn case_insensitive_matching_treats_differently_cased_names_as_same_file() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/Photo.JPG"), b"same-bytes-len");
        write(&dir.path().join("Daten_1b/photo.jpg"), b"same-bytes-len");

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();

        assert_eq!(tree.root.children.len(), 1);
        let node = &tree.root.children[0];
        assert!(node.clones.iter().all(|c| c.state == MatchState::Present));
    }

    #[test]
    fn empty_folder_rolls_up_as_empty_not_missing() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1a/empty_dir")).unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1b")).unwrap();

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();

        let empty_dir = tree.root.children.iter().find(|n| n.name == "empty_dir").unwrap();
        let rollup = empty_dir.rollup.as_ref().unwrap();
        assert!(rollup.iter().all(|r| r.state == RollupState::Empty));
    }
}
