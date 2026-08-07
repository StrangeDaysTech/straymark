//! `straymark followups install-merge-driver` (GH #391 follow-up).
//!
//! The merge driver shipped in cli-3.41.0 but stayed inert everywhere, because
//! taking effect needs two per-clone edits nobody ran. These tests pin the
//! behaviour that makes running it safe to repeat and safe to run on a repo
//! that already has opinions about its own git config.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

/// A git repo with a minimal StrayMark installation.
fn project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path();
    Command::new("git").current_dir(p).args(["init", "-q"]).status().unwrap();
    std::fs::create_dir_all(p.join(".straymark")).unwrap();
    std::fs::write(p.join(".straymark/config.yml"), "language: en\n").unwrap();
    std::fs::write(p.join("STRAYMARK.md"), "# rules\n").unwrap();
    tmp
}

fn git_config(dir: &std::path::Path, key: &str) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["config", "--get", key])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn wires_both_halves_and_is_idempotent() {
    let tmp = project();
    let p = tmp.path();

    cargo_bin_cmd!("straymark")
        .args(["followups", "install-merge-driver"])
        .arg(p)
        .assert()
        .success();

    let attrs = std::fs::read_to_string(p.join(".gitattributes")).unwrap();
    assert!(
        attrs.contains(".straymark/follow-ups-backlog.md merge=straymark-followups"),
        ".gitattributes must bind the registry to the driver"
    );
    assert_eq!(
        git_config(p, "merge.straymark-followups.driver"),
        "straymark followups merge-driver %O %A %B"
    );

    // Re-running must not append a second attribute line — the file is the
    // adopter's, and a command that grows it on every run is unusable in an
    // `init` prompt people may accept more than once.
    cargo_bin_cmd!("straymark")
        .args(["followups", "install-merge-driver"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("already wired"));

    let attrs_again = std::fs::read_to_string(p.join(".gitattributes")).unwrap();
    assert_eq!(attrs, attrs_again, "second run must change nothing");
    assert_eq!(
        attrs.matches("merge=straymark-followups").count(),
        1,
        "exactly one binding"
    );
}

#[test]
fn preserves_an_existing_gitattributes_file() {
    let tmp = project();
    let p = tmp.path();
    // No trailing newline: the append must not glue itself onto the last line.
    std::fs::write(p.join(".gitattributes"), "*.rs text eol=lf\n*.png binary").unwrap();

    cargo_bin_cmd!("straymark")
        .args(["followups", "install-merge-driver"])
        .arg(p)
        .assert()
        .success();

    let attrs = std::fs::read_to_string(p.join(".gitattributes")).unwrap();
    assert!(attrs.contains("*.rs text eol=lf"), "existing rules survive");
    assert!(attrs.contains("*.png binary\n"), "last line stays intact");
    assert!(attrs.contains("\n.straymark/follow-ups-backlog.md merge=straymark-followups"));
}

/// An adopter who pointed the registry at their own driver made a deliberate
/// choice. Overwriting it silently would be the tool deciding it knows better.
#[test]
fn respects_a_foreign_driver_already_configured() {
    let tmp = project();
    let p = tmp.path();
    std::fs::write(
        p.join(".gitattributes"),
        ".straymark/follow-ups-backlog.md merge=my-own-driver\n",
    )
    .unwrap();
    Command::new("git")
        .current_dir(p)
        .args(["config", "merge.straymark-followups.driver", "my-own-command %A"])
        .status()
        .unwrap();

    cargo_bin_cmd!("straymark")
        .args(["followups", "install-merge-driver"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("already set to something else"));

    assert_eq!(
        git_config(p, "merge.straymark-followups.driver"),
        "my-own-command %A",
        "a foreign driver command must survive untouched"
    );
    let attrs = std::fs::read_to_string(p.join(".gitattributes")).unwrap();
    assert!(!attrs.contains("merge=straymark-followups"));
    assert!(attrs.contains("merge=my-own-driver"));
}

#[test]
fn refuses_outside_a_git_repository() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
    std::fs::write(tmp.path().join(".straymark/config.yml"), "language: en\n").unwrap();

    cargo_bin_cmd!("straymark")
        .args(["followups", "install-merge-driver"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}

/// `init` runs unattended in CI and provisioning scripts. A prompt that blocks
/// there is worse than the friction it saves, so the decline flag has to exist
/// and the prompt must never appear without a TTY.
#[test]
fn init_accepts_the_non_interactive_decline_flag() {
    cargo_bin_cmd!("straymark")
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--merge-driver"))
        .stdout(predicate::str::contains("--no-merge-driver"));
}
