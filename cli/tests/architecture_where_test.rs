//! Integration tests for `straymark status --where` (Loom A1.4, Spec 002 §8).
//!
//! The headline test is the **consistency gate** (T4.3 / spec §11.5 textual
//! half, NFR3): the components the projection flags `active`/`in-progress` must
//! line up with `straymark charter list --status in-progress` + `charter
//! drift`, and `implemented` with `charter list --status closed`. Both views are
//! driven by the same `core` extractors, so the test asserts they agree on one
//! fixture corpus.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) {
    let status = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git invocation failed");
    assert!(status.success(), "git {} failed", args.join(" "));
}

const MODEL_YML: &str = r#"version: 0
layers:
  - { id: "core", label: "Core", order: 0 }
components:
  - id: "auth"
    label: "Auth"
    layer: "core"
    globs: ["src/auth/**"]
    links: []
    docs: []
    external: false
  - id: "billing"
    label: "Billing"
    layer: "core"
    globs: ["src/billing/**"]
    links: []
    docs: []
    external: false
"#;

/// Write a Charter with a given status and declared files.
fn write_charter(dir: &Path, num: u32, id: &str, status: &str, title: &str, declared: &[&str]) {
    let charters = dir.join(".straymark").join("charters");
    std::fs::create_dir_all(&charters).unwrap();
    let mut content = format!(
        "---\ncharter_id: {id}\nstatus: {status}\neffort_estimate: M\ntrigger: \"t\"\n---\n\n# Charter: {title}\n\n## Files to modify\n\n| File | Change |\n|---|---|\n",
    );
    for f in declared {
        content.push_str(&format!("| `{f}` | edit |\n"));
    }
    content.push_str("\n## Tasks\n\n1. Do it.\n");
    std::fs::write(charters.join(format!("{num:02}-{title}.md")), content).unwrap();
}

/// A fixture project: model with `auth` + `billing`, an in-progress Charter on
/// auth, a closed Charter on billing, both source files on disk, git-initialized
/// with an uncommitted edit inside auth (so it reads as in-progress).
fn setup_fixture(dir: &Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("architecture")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(straymark.join("architecture").join("model.yml"), MODEL_YML).unwrap();

    write_charter(dir, 1, "CHARTER-01", "in-progress", "auth-work", &["src/auth/login.rs"]);
    write_charter(dir, 2, "CHARTER-02", "closed", "billing-done", &["src/billing/charge.rs"]);

    std::fs::create_dir_all(dir.join("src/auth")).unwrap();
    std::fs::create_dir_all(dir.join("src/billing")).unwrap();
    std::fs::write(dir.join("src/auth/login.rs"), "// v1\n").unwrap();
    std::fs::write(dir.join("src/billing/charge.rs"), "// v1\n").unwrap();

    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "initial"]);
    // Uncommitted edit inside auth → declared ∩ git-modified → in-progress.
    std::fs::write(dir.join("src/auth/login.rs"), "// v2 edited\n").unwrap();
}

#[test]
fn where_degrades_without_model() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".straymark")).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["status", "--where"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No architecture model"))
        .stdout(predicate::str::contains("architecture generate"));
}

#[test]
fn where_marks_active_in_progress_and_implemented() {
    let dir = TempDir::new().unwrap();
    setup_fixture(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["status", "--where"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        // auth is the active component, with an already-touched declared file.
        .stdout(predicate::str::contains("Auth"))
        .stdout(predicate::str::contains("[active]"))
        .stdout(predicate::str::contains("[in-progress]"))
        .stdout(predicate::str::contains("you are here"))
        // billing is implemented (closed Charter).
        .stdout(predicate::str::contains("[implemented]"));
}

/// The consistency gate (T4.3): the projection's `active` component corresponds
/// to the Charter `charter list --status in-progress` reports, and `implemented`
/// to `--status closed`. Driven by the same `core` extractors, the two views
/// must not disagree.
#[test]
fn where_is_consistent_with_charter_list() {
    let dir = TempDir::new().unwrap();
    setup_fixture(dir.path());
    let path = dir.path().to_str().unwrap();

    // in-progress Charters → exactly the auth Charter.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "list", "--status", "in-progress", path])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth-work"))
        .stdout(predicate::str::contains("billing-done").not());

    // closed Charters → exactly the billing Charter.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "list", "--status", "closed", path])
        .assert()
        .success()
        .stdout(predicate::str::contains("billing-done"))
        .stdout(predicate::str::contains("auth-work").not());

    // The projection agrees: auth active (the in-progress Charter's component),
    // billing implemented (the closed Charter's component).
    let out = Command::cargo_bin("straymark")
        .unwrap()
        .args(["status", "--where"])
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);

    // Locate each component's line and check its badges.
    let auth_line = text
        .lines()
        .find(|l| l.contains("Auth"))
        .expect("auth component line present");
    assert!(auth_line.contains("[active]"), "auth must be active: {auth_line}");

    let billing_line = text
        .lines()
        .find(|l| l.contains("Billing"))
        .expect("billing component line present");
    assert!(
        billing_line.contains("[implemented]"),
        "billing must be implemented: {billing_line}"
    );
    assert!(
        !billing_line.contains("[active]"),
        "billing must NOT be active: {billing_line}"
    );
}
