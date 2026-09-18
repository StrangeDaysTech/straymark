//! The one rule every project-wide directory walker shares.
//!
//! Walkers skip a fixed list of names (`target`, `node_modules`, …), but a
//! project can also hold *another checkout of itself*: a linked git worktree
//! under `.worktrees/`, a submodule, a nested clone. Reading it duplicates the
//! project — `architecture generate` invented a `.worktrees` component holding
//! a full copy of an adopter's repo (#434). The marker is structural, so no
//! gitignore parsing and nothing for an adopter to configure.

use std::path::Path;

/// True when `dir` is the root of another checkout: a linked worktree or a
/// submodule (a `.git` *file*) or a nested clone (a `.git` directory). Call it
/// on sub-directories only — the project root itself has a `.git`.
pub fn is_nested_checkout(dir: &Path) -> bool {
    dir.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_file_or_dir_marks_a_nested_checkout() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree = tmp.path().join(".worktrees/feature");
        let clone = tmp.path().join("vendor/clone");
        let plain = tmp.path().join("src");
        for d in [&worktree, &clone, &plain] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(worktree.join(".git"), "gitdir: /elsewhere\n").unwrap();
        std::fs::create_dir(clone.join(".git")).unwrap();
        assert!(is_nested_checkout(&worktree));
        assert!(is_nested_checkout(&clone));
        assert!(!is_nested_checkout(&plain));
    }
}
