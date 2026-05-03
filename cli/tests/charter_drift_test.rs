//! Integration tests for `devtrail charter drift`. Requires a real git repo
//! and `bash` in PATH; tests are skipped on Windows runners that lack bash.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Inline copy of the framework's check-charter-drift.sh — the integration
/// path expects the script under `.devtrail/scripts/`.
const DRIFT_SCRIPT: &str = include_str!("../../dist/.devtrail/scripts/check-charter-drift.sh");

const CHARTER_TEMPLATE: &str = r#"---
charter_id: CHARTER-NN
status: declared
effort_estimate: M
trigger: "[1-line]"
---

# Charter: [BRIEF TITLE]

## Files to modify

| File | Change |
|---|---|

## Tasks

1. Sync.
"#;

fn bash_available() -> bool {
    StdCommand::new("bash")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn setup_devtrail(dir: &Path) {
    let devtrail = dir.join(".devtrail");
    std::fs::create_dir_all(devtrail.join("templates")).unwrap();
    std::fs::create_dir_all(devtrail.join("scripts")).unwrap();
    std::fs::create_dir_all(devtrail.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::write(devtrail.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        devtrail.join("templates").join("charter-template.md"),
        CHARTER_TEMPLATE,
    )
    .unwrap();
    let script_path = devtrail.join("scripts").join("check-charter-drift.sh");
    std::fs::write(&script_path, DRIFT_SCRIPT).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).unwrap();
    }
}

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

fn write_charter_with_files(dir: &Path, declared: &[&str], originating_ailog: Option<&str>) {
    let charters_dir = dir.join("docs").join("charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    let mut content = String::from("---\ncharter_id: CHARTER-01\nstatus: declared\neffort_estimate: M\ntrigger: \"test trigger\"\n");
    if let Some(ailog) = originating_ailog {
        content.push_str(&format!("originating_ailogs:\n  - {}\n", ailog));
    }
    content.push_str("---\n\n# Charter: Drift Test\n\n## Files to modify\n\n| File | Change |\n|---|---|\n");
    for f in declared {
        content.push_str(&format!("| `{}` | edit |\n", f));
    }
    content.push_str("\n## Tasks\n\n1. Run.\n");
    std::fs::write(charters_dir.join("01-drift-test.md"), content).unwrap();
}

#[test]
fn charter_drift_clean_when_declared_matches_modified() {
    if !bash_available() {
        eprintln!("skipping: bash not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter_with_files(dir.path(), &["src/foo.rs"], None);
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/foo.rs"), "// initial\n").unwrap();

    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "initial"]);
    std::fs::write(dir.path().join("src/foo.rs"), "// edited\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "edit foo"]);

    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "drift", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("OK No drift detected"));
}

#[test]
fn charter_drift_detects_declared_but_not_modified() {
    if !bash_available() {
        eprintln!("skipping: bash not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter_with_files(dir.path(), &["src/foo.rs", "src/bar.rs"], None);
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/foo.rs"), "// initial\n").unwrap();
    std::fs::write(dir.path().join("src/bar.rs"), "// initial\n").unwrap();

    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "initial"]);
    // Only modify foo, not bar — bar is the drift.
    std::fs::write(dir.path().join("src/foo.rs"), "// edited\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "edit foo only"]);

    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "drift", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure() // exit 1
        .stdout(predicate::str::contains("Declared in Charter but NOT modified"))
        .stdout(predicate::str::contains("src/bar.rs"));
}

#[test]
fn charter_drift_ailog_suppression_clears_documented_paths() {
    if !bash_available() {
        eprintln!("skipping: bash not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    // Write an AILOG that documents the drift path under ## Risk.
    let ailog_path = dir
        .path()
        .join(".devtrail/07-ai-audit/agent-logs/AILOG-2026-05-02-001-document-bar.md");
    std::fs::write(
        &ailog_path,
        r#"---
id: AILOG-2026-05-02-001
title: Document bar
agent: test
confidence: high
risk_level: low
review_required: false
---

# AILOG: Document bar

## Context

Some context.

## Risk

- **R3 (new, not in Charter)**: `src/bar.rs` was declared but not touched
  because the implementation simplified scope mid-flight.

## Outcome

Done.
"#,
    )
    .unwrap();

    write_charter_with_files(
        dir.path(),
        &["src/foo.rs", "src/bar.rs"],
        Some("AILOG-2026-05-02-001"),
    );
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/foo.rs"), "// initial\n").unwrap();
    std::fs::write(dir.path().join("src/bar.rs"), "// initial\n").unwrap();

    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "initial"]);
    std::fs::write(dir.path().join("src/foo.rs"), "// edited\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "edit foo only"]);

    // Default behavior: AILOG suppression kicks in, drift is cleared.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "drift", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("AILOG-suppressed"))
        .stdout(predicate::str::contains("AILOG-2026-05-02-001"));
}

#[test]
fn charter_drift_no_ailog_suppress_disables_suppression() {
    if !bash_available() {
        eprintln!("skipping: bash not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    let ailog_path = dir
        .path()
        .join(".devtrail/07-ai-audit/agent-logs/AILOG-2026-05-02-001-document-bar.md");
    std::fs::write(
        &ailog_path,
        r#"---
id: AILOG-2026-05-02-001
title: Document bar
agent: test
confidence: high
risk_level: low
review_required: false
---

## Risk

- **R3**: `src/bar.rs` documented.
"#,
    )
    .unwrap();

    write_charter_with_files(
        dir.path(),
        &["src/foo.rs", "src/bar.rs"],
        Some("AILOG-2026-05-02-001"),
    );
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/foo.rs"), "// initial\n").unwrap();
    std::fs::write(dir.path().join("src/bar.rs"), "// initial\n").unwrap();

    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "initial"]);
    std::fs::write(dir.path().join("src/foo.rs"), "// edited\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "edit foo only"]);

    // With --no-ailog-suppress, drift is reported even though it's documented.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "drift",
            "CHARTER-01",
            "--no-ailog-suppress",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("src/bar.rs"))
        .stdout(predicate::str::contains("Declared in Charter but NOT modified"));
}

#[test]
fn charter_drift_ignores_path_references_in_change_column() {
    // F3 (cli-3.7.1 / fw-4.6.1): the drift script's regex used to extract
    // backtick-quoted paths from ANY column of the table, including the
    // "Change" column. A path mentioned as a textual reference (e.g. "follows
    // the pattern of `docs/plans/README.md`") would be parsed as a declared
    // deliverable → false-positive omission warning. This test pins the fix:
    // a Charter that mentions `docs/plans/README.md` in column 2 should
    // declare only the column-1 paths.
    if !bash_available() {
        eprintln!("skipping: bash not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    // Build a Charter where the "Change" column contains a backtick-quoted path.
    let charters_dir = dir.path().join("docs").join("charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    let charter = r#"---
charter_id: CHARTER-01
status: declared
effort_estimate: S
trigger: "test"
---

# Charter: F3 cross-reference test

## Files to modify

| File | Change |
|---|---|
| `src/foo.go` | edit (follows the pattern of `docs/plans/README.md`) |
| `src/bar.go` | edit |

## Tasks

1. Run.
"#;
    std::fs::write(charters_dir.join("01-cross-ref.md"), charter).unwrap();

    // Create only the column-1 files, modify them. Don't touch
    // docs/plans/README.md — if F3 is fixed, the script must NOT flag it
    // as declared-but-not-modified.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("docs/plans")).unwrap();
    std::fs::write(dir.path().join("src/foo.go"), "// initial\n").unwrap();
    std::fs::write(dir.path().join("src/bar.go"), "// initial\n").unwrap();
    std::fs::write(dir.path().join("docs/plans/README.md"), "# plans\n").unwrap();

    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "initial"]);
    std::fs::write(dir.path().join("src/foo.go"), "// edited\n").unwrap();
    std::fs::write(dir.path().join("src/bar.go"), "// edited\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "edit foo and bar"]);

    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "drift", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("OK No drift detected"));
}
