//! GH #399: Qoder CLI support.
//!
//! Two guarantees are pinned down here:
//!
//! 1. `dist/.qoder/skills/` is a byte-for-byte copy of `dist/.claude/skills/`
//!    (Qoder consumes the full Claude-format SKILL.md frontmatter). If the two
//!    trees drift, adopters get stale skills on one agent.
//! 2. `straymark install-skills --agent qoder` installs the project's
//!    `.qoder/skills/` into `$QODER_CONFIG_DIR/skills/`.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn dist_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("dist")
}

fn walk_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.insert(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    out
}

#[test]
fn qoder_skills_mirror_claude_skills() {
    let claude = dist_root().join(".claude").join("skills");
    let qoder = dist_root().join(".qoder").join("skills");
    assert!(claude.is_dir(), "missing {}", claude.display());
    assert!(qoder.is_dir(), "missing {}", qoder.display());

    let claude_files = walk_files(&claude);
    let qoder_files = walk_files(&qoder);
    assert_eq!(
        claude_files, qoder_files,
        "dist/.qoder/skills/ must mirror dist/.claude/skills/ file-for-file"
    );
    assert!(!qoder_files.is_empty());

    for rel in &qoder_files {
        let a = std::fs::read(claude.join(rel)).unwrap();
        let b = std::fs::read(qoder.join(rel)).unwrap();
        assert_eq!(a, b, "content drift in {}", rel.display());
    }
}

#[test]
fn install_skills_qoder_uses_qoder_config_dir() {
    let project = TempDir::new().unwrap();
    let qoder_home = TempDir::new().unwrap();

    let skill_dir = project
        .path()
        .join(".qoder")
        .join("skills")
        .join("straymark-foo");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: straymark-foo\ndescription: test skill\n---\n# Foo\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .args(["install-skills", "--agent", "qoder", "--path"])
        .arg(project.path().to_str().unwrap())
        .env("QODER_CONFIG_DIR", qoder_home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1 skill(s) installed"))
        .stdout(predicate::str::contains("qoder will discover them"));

    let installed = qoder_home.path().join("skills").join("straymark-foo").join("SKILL.md");
    assert!(installed.exists(), "expected {}", installed.display());
    assert_eq!(
        std::fs::read_to_string(&installed).unwrap(),
        "---\nname: straymark-foo\ndescription: test skill\n---\n# Foo\n"
    );
}
