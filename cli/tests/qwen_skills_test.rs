//! Qwen Code CLI support.
//!
//! Two guarantees are pinned down here:
//!
//! 1. `dist/.qwen/skills/` is a byte-for-byte copy of `dist/.claude/skills/`
//!    (Qwen Code parses the full Claude-format SKILL.md frontmatter, including
//!    `allowed-tools`). If the two trees drift, adopters get stale skills on
//!    one agent.
//! 2. `straymark install-skills --agent qwen` installs the project's
//!    `.qwen/skills/` into `$QWEN_HOME/skills/` — the same directory Qwen
//!    Code's own `Storage.getGlobalQwenDir()` resolves.

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
fn qwen_skills_mirror_claude_skills() {
    let claude = dist_root().join(".claude").join("skills");
    let qwen = dist_root().join(".qwen").join("skills");
    assert!(claude.is_dir(), "missing {}", claude.display());
    assert!(qwen.is_dir(), "missing {}", qwen.display());

    let claude_files = walk_files(&claude);
    let qwen_files = walk_files(&qwen);
    assert_eq!(
        claude_files, qwen_files,
        "dist/.qwen/skills/ must mirror dist/.claude/skills/ file-for-file"
    );
    assert!(!qwen_files.is_empty());

    for rel in &qwen_files {
        let a = std::fs::read(claude.join(rel)).unwrap();
        let b = std::fs::read(qwen.join(rel)).unwrap();
        assert_eq!(a, b, "content drift in {}", rel.display());
    }
}

#[test]
fn install_skills_qwen_uses_qwen_home() {
    let project = TempDir::new().unwrap();
    let qwen_home = TempDir::new().unwrap();

    let skill_dir = project
        .path()
        .join(".qwen")
        .join("skills")
        .join("straymark-foo");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: straymark-foo\ndescription: test skill\n---\n# Foo\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .args(["install-skills", "--agent", "qwen", "--path"])
        .arg(project.path().to_str().unwrap())
        .env("QWEN_HOME", qwen_home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1 skill(s) installed"))
        .stdout(predicate::str::contains("qwen will discover them"));

    let installed = qwen_home.path().join("skills").join("straymark-foo").join("SKILL.md");
    assert!(installed.exists(), "expected {}", installed.display());
    assert_eq!(
        std::fs::read_to_string(&installed).unwrap(),
        "---\nname: straymark-foo\ndescription: test skill\n---\n# Foo\n"
    );
}

/// `validate --agent` has its own clap `value_parser`, separate from the one on
/// `install-skills`. Widening only the dispatch inside `validate.rs` leaves the
/// flag rejected at parse time — the exact gap this test pins down.
#[test]
fn validate_accepts_every_user_level_agent() {
    for agent in ["codex", "qoder", "qwen"] {
        let home = TempDir::new().unwrap();
        // Point every home-resolution env var at an empty dir so the command
        // reaches its "skills directory not found" branch instead of touching
        // the developer's real installation.
        cargo_bin_cmd!("straymark")
            .args(["validate", "--agent", agent])
            .env("CODEX_HOME", home.path())
            .env("QODER_CONFIG_DIR", home.path())
            .env("QWEN_HOME", home.path())
            .assert()
            .failure()
            .stdout(predicate::str::contains("skills directory not found"))
            .stderr(predicate::str::contains("invalid value").not());
    }
}

/// The directive target has to be declared in the manifest, or `init` /
/// `update` / `repair` never write `QWEN.md` — Qwen Code's default context
/// filename is `QWEN.md`, so without it the agent loads no governance at all.
#[test]
fn manifest_declares_qwen_surface() {
    let manifest = std::fs::read_to_string(dist_root().join("dist-manifest.yml")).unwrap();
    assert!(
        manifest.contains("- .qwen/skills/"),
        "dist-manifest.yml must ship .qwen/skills/"
    );
    assert!(
        manifest.contains("target: QWEN.md"),
        "dist-manifest.yml must declare the QWEN.md injection"
    );
    assert!(
        manifest.contains("template: dist-templates/directives/QWEN.md"),
        "the QWEN.md injection must point at its template"
    );
    assert!(
        dist_root()
            .join("dist-templates/directives/QWEN.md")
            .is_file(),
        "the QWEN.md directive template must exist"
    );
}
