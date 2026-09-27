use thiserror::Error;

use crate::batch::action::ActionKind;
use crate::tree::{EntryKind, MatchState, MergedNode};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("no such path in this drive group")]
    NotFound,
    #[error("this path is a folder, not a file")]
    NotAFile,
    #[error("this file doesn't exist in any clone")]
    NothingToSync,
    #[error("this file is already in sync across all clones")]
    AlreadyInSync,
    #[error("this file has conflicting versions and needs the conflict wizard")]
    HasConflict,
    #[error("this file doesn't exist in any clone, nothing to delete")]
    NothingToDelete,
}

/// Finds the node at `rel_path` within `root` (the tree root itself for `""`).
pub fn find_node<'a>(root: &'a MergedNode, rel_path: &str) -> Option<&'a MergedNode> {
    if root.rel_path == rel_path {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_node(c, rel_path))
}

/// Plans a plain "sync this file to all clones" action. Only valid when the
/// file has no `Differs` clones — a real conflict (differing content among
/// clones that *have* the file) always requires the explicit conflict
/// wizard (see `plan_resolve_conflict`), never an automatic majority vote.
pub fn plan_sync_file(node: &MergedNode) -> Result<ActionKind, PlanError> {
    if node.kind != EntryKind::File {
        return Err(PlanError::NotAFile);
    }
    if node.clones.iter().any(|c| c.state == MatchState::Differs) {
        return Err(PlanError::HasConflict);
    }
    let Some(source) = node.clones.iter().find(|c| c.state == MatchState::Present) else {
        return Err(PlanError::NothingToSync);
    };
    let targets: Vec<String> = node
        .clones
        .iter()
        .filter(|c| c.state == MatchState::Missing)
        .map(|c| c.clone.clone())
        .collect();
    if targets.is_empty() {
        return Err(PlanError::AlreadyInSync);
    }
    Ok(ActionKind::SyncFile {
        rel_path: node.rel_path.clone(),
        source_clone: source.clone.clone(),
        target_clones: targets,
    })
}

/// Plans a "delete this file in all clones that have it" action.
pub fn plan_delete_file(node: &MergedNode) -> Result<ActionKind, PlanError> {
    if node.kind != EntryKind::File {
        return Err(PlanError::NotAFile);
    }
    let clones: Vec<String> = node
        .clones
        .iter()
        .filter(|c| c.state != MatchState::Missing)
        .map(|c| c.clone.clone())
        .collect();
    if clones.is_empty() {
        return Err(PlanError::NothingToDelete);
    }
    Ok(ActionKind::DeleteFile {
        rel_path: node.rel_path.clone(),
        clones,
    })
}

/// Result of expanding a folder-level bulk action into individual file actions.
pub struct FolderPlan {
    pub actions: Vec<ActionKind>,
    /// rel_paths of files that have real conflicts and were skipped — the
    /// caller should surface these so the user can resolve them individually
    /// via the conflict wizard.
    pub skipped_conflicts: Vec<String>,
}

fn visit_files<'a>(node: &'a MergedNode, f: &mut impl FnMut(&'a MergedNode)) {
    match node.kind {
        EntryKind::File => f(node),
        EntryKind::Folder => {
            for child in &node.children {
                visit_files(child, f);
            }
        }
    }
}

/// Expands "sync this folder to all clones" into one `SyncFile` action per
/// file that's unambiguously fixable (missing from some clones, identical
/// everywhere it exists). Files with real conflicts are left out of
/// `actions` and listed in `skipped_conflicts` instead.
pub fn plan_sync_folder(node: &MergedNode) -> FolderPlan {
    let mut actions = Vec::new();
    let mut skipped_conflicts = Vec::new();
    visit_files(node, &mut |file| match plan_sync_file(file) {
        Ok(action) => actions.push(action),
        Err(PlanError::HasConflict) => skipped_conflicts.push(file.rel_path.clone()),
        Err(_) => {}
    });
    FolderPlan {
        actions,
        skipped_conflicts,
    }
}

/// Expands "delete this folder in all clones" into one `DeleteFile` action
/// per file currently present in at least one clone.
pub fn plan_delete_folder(node: &MergedNode) -> Vec<ActionKind> {
    let mut actions = Vec::new();
    visit_files(node, &mut |file| {
        if let Ok(action) = plan_delete_file(file) {
            actions.push(action);
        }
    });
    actions
}

/// Plans the explicit conflict-wizard resolution: sync `chosen_clone`'s
/// version to every other clone whose value differs from it (comparing
/// directly against the chosen clone, not the tree's majority-vote
/// reference — the whole point of the wizard is overriding that).
pub fn plan_resolve_conflict(
    node: &MergedNode,
    chosen_clone: &str,
) -> Result<ActionKind, PlanError> {
    if node.kind != EntryKind::File {
        return Err(PlanError::NotAFile);
    }
    let Some(source) = node.clones.iter().find(|c| c.clone == chosen_clone) else {
        return Err(PlanError::NotFound);
    };
    if source.state == MatchState::Missing {
        return Err(PlanError::NotFound);
    }
    let targets: Vec<String> = node
        .clones
        .iter()
        .filter(|c| c.clone != chosen_clone)
        .filter(|c| c.size != source.size || c.hash != source.hash)
        .map(|c| c.clone.clone())
        .collect();
    if targets.is_empty() {
        return Err(PlanError::AlreadyInSync);
    }
    Ok(ActionKind::SyncFile {
        rel_path: node.rel_path.clone(),
        source_clone: chosen_clone.to_string(),
        target_clones: targets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ComparisonMode;
    use crate::drive::scan_drives;
    use crate::tree::build_merged_tree;
    use tempfile::tempdir;

    fn write(path: &std::path::Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn plans_sync_for_missing_only_file() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"x");
        write(&dir.path().join("Daten_1b/f.txt"), b"x");
        // Give clone c a mounted drive with a different file, so it's part of
        // the group but genuinely missing f.txt (not just absent from disk).
        write(&dir.path().join("Daten_1c/other.txt"), b"y");

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();
        let node = find_node(&tree.root, "f.txt").unwrap();

        let action = plan_sync_file(node).unwrap();
        match action {
            ActionKind::SyncFile {
                source_clone,
                target_clones,
                ..
            } => {
                assert!(source_clone == "a" || source_clone == "b");
                assert_eq!(target_clones, vec!["c".to_string()]);
            }
            _ => panic!("expected SyncFile"),
        }
    }

    #[test]
    fn refuses_plain_sync_when_conflict_present() {
        let dir = tempdir().unwrap();
        // Two clones agree on size (basic mode's blind spot), a third has a
        // genuinely different size, which is enough to force a real Differs
        // state and make this an ambiguous conflict.
        write(&dir.path().join("Daten_1a/f.txt"), b"AAAA");
        write(&dir.path().join("Daten_1b/f.txt"), b"BBBB");
        write(&dir.path().join("Daten_1c/f.txt"), b"C");

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();
        let node = find_node(&tree.root, "f.txt").unwrap();

        assert_eq!(plan_sync_file(node), Err(PlanError::HasConflict));
    }

    #[test]
    fn plans_delete_for_all_present_clones() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"x");
        write(&dir.path().join("Daten_1b/f.txt"), b"y");

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();
        let node = find_node(&tree.root, "f.txt").unwrap();

        let action = plan_delete_file(node).unwrap();
        match action {
            ActionKind::DeleteFile { clones, .. } => {
                let mut clones = clones;
                clones.sort();
                assert_eq!(clones, vec!["a".to_string(), "b".to_string()]);
            }
            _ => panic!("expected DeleteFile"),
        }
    }

    #[test]
    fn folder_sync_skips_conflicts_but_queues_unambiguous_files() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/docs/ok.txt"), b"x");
        // ok.txt missing from b: unambiguous, should be queued.
        write(&dir.path().join("Daten_1a/docs/conflict.txt"), b"A");
        write(&dir.path().join("Daten_1b/docs/conflict.txt"), b"BB");
        // conflict.txt differs in size between a and b: real conflict, should be skipped.

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();
        let docs = find_node(&tree.root, "docs").unwrap();

        let plan = plan_sync_folder(docs);
        assert_eq!(
            plan.skipped_conflicts,
            vec!["docs/conflict.txt".to_string()]
        );
        assert_eq!(plan.actions.len(), 1);
        match &plan.actions[0] {
            ActionKind::SyncFile { rel_path, .. } => assert_eq!(rel_path, "docs/ok.txt"),
            _ => panic!("expected SyncFile"),
        }
    }

    #[test]
    fn resolve_conflict_targets_only_clones_differing_from_chosen_source() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"AAAA");
        write(&dir.path().join("Daten_1b/f.txt"), b"BBBB"); // same size as a, different content — basic mode can't tell
        write(&dir.path().join("Daten_1c/f.txt"), b"CCCCCCCC"); // different size

        let groups = scan_drives(dir.path()).unwrap();
        let group = groups.into_iter().find(|g| g.key.name == "Daten").unwrap();
        let tree = build_merged_tree(&group, ComparisonMode::NameSize, None).unwrap();
        let node = find_node(&tree.root, "f.txt").unwrap();

        // Chosen clone "a" as source: "c" differs in size, "b" matches by size (basic mode blind spot, expected).
        let action = plan_resolve_conflict(node, "a").unwrap();
        match action {
            ActionKind::SyncFile {
                source_clone,
                target_clones,
                ..
            } => {
                assert_eq!(source_clone, "a");
                assert_eq!(target_clones, vec!["c".to_string()]);
            }
            _ => panic!("expected SyncFile"),
        }
    }
}
