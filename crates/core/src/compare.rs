use std::collections::HashMap;

use crate::config::ComparisonMode;
use crate::tree::MatchState;

/// Per-clone metadata for a single file, as observed while walking that clone's tree.
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime_unix: Option<i64>,
    /// Only populated when the active `ComparisonMode` requires content hashing.
    pub hash: Option<u64>,
}

/// The "reference" value a file's clones are compared against, purely for
/// coloring the merged tree. It is the most common (size[, hash]) among the
/// clones that have the file, tie-broken by clone order (earliest wins).
/// This is provisional and never implies a chosen sync source of truth —
/// that choice is always explicit, via the conflict wizard.
fn reference_key(mode: ComparisonMode, meta: &FileMeta) -> (u64, Option<u64>) {
    match mode {
        ComparisonMode::NameSize => (meta.size, None),
        ComparisonMode::NameSizeHash => (meta.size, meta.hash),
    }
}

/// `present` lists clones (in group/display order) that have this file, with their metadata.
/// Returns the reference key that the majority of present clones agree on.
pub fn compute_reference(
    mode: ComparisonMode,
    present: &[(&str, &FileMeta)],
) -> Option<(u64, Option<u64>)> {
    if present.is_empty() {
        return None;
    }
    let mut counts: HashMap<(u64, Option<u64>), usize> = HashMap::new();
    for (_, meta) in present {
        *counts.entry(reference_key(mode, meta)).or_insert(0) += 1;
    }
    // Tie-break by clone order: scan `present` in order and keep the first
    // key whose count matches the maximum.
    let max_count = counts.values().copied().max().unwrap_or(0);
    present
        .iter()
        .map(|(_, meta)| reference_key(mode, meta))
        .find(|key| counts.get(key).copied().unwrap_or(0) == max_count)
}

/// Determines a single clone's `MatchState` for a file given the group's reference value.
pub fn match_state(
    mode: ComparisonMode,
    meta: Option<&FileMeta>,
    reference: Option<(u64, Option<u64>)>,
) -> MatchState {
    let Some(meta) = meta else {
        return MatchState::Missing;
    };
    let Some(reference) = reference else {
        return MatchState::Present;
    };
    if reference_key(mode, meta) == reference {
        MatchState::Present
    } else {
        MatchState::Differs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(size: u64, hash: Option<u64>) -> FileMeta {
        FileMeta {
            size,
            mtime_unix: None,
            hash,
        }
    }

    #[test]
    fn majority_size_wins_reference_in_basic_mode() {
        let a = meta(100, None);
        let b = meta(100, None);
        let c = meta(200, None);
        let present = vec![("a", &a), ("b", &b), ("c", &c)];
        let reference = compute_reference(ComparisonMode::NameSize, &present);
        assert_eq!(reference, Some((100, None)));
        assert_eq!(
            match_state(ComparisonMode::NameSize, Some(&a), reference),
            MatchState::Present
        );
        assert_eq!(
            match_state(ComparisonMode::NameSize, Some(&c), reference),
            MatchState::Differs
        );
    }

    #[test]
    fn tie_breaks_by_clone_order() {
        let a = meta(100, None);
        let b = meta(200, None);
        let present = vec![("a", &a), ("b", &b)];
        // 1-1 tie: first clone in order (a) wins.
        let reference = compute_reference(ComparisonMode::NameSize, &present);
        assert_eq!(reference, Some((100, None)));
    }

    #[test]
    fn hash_mode_distinguishes_same_size_different_content() {
        let a = meta(100, Some(111));
        let b = meta(100, Some(111));
        let c = meta(100, Some(999));
        let present = vec![("a", &a), ("b", &b), ("c", &c)];
        let reference = compute_reference(ComparisonMode::NameSizeHash, &present);
        assert_eq!(
            match_state(ComparisonMode::NameSizeHash, Some(&c), reference),
            MatchState::Differs
        );
        assert_eq!(
            match_state(ComparisonMode::NameSizeHash, Some(&a), reference),
            MatchState::Present
        );
    }

    #[test]
    fn missing_clone_is_missing_regardless_of_reference() {
        assert_eq!(
            match_state(ComparisonMode::NameSize, None, Some((100, None))),
            MatchState::Missing
        );
    }
}
