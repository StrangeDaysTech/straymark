//! Integration tests for `devtrail charter close`. Only the
//! `--from-template --non-interactive` path is testable without a TTY; the
//! interactive flow is exercised manually and via the unit tests in
//! `cli/src/commands/charter/close.rs`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

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

const TELEMETRY_TEMPLATE: &str = r#"# DevTrail Charter telemetry — fill at Charter close.
charter_telemetry:
  charter_id: "CHARTER-NN"
  charter_title: "<short title>"
  closed_at: "YYYY-MM-DD"
  effort:
    estimated_effort: "M (~1.5h)"
    actual_effort: "M (~1.5h)"
  outcome:
    completed_as_planned: true
    scope_changes: "ninguno"
"#;

const TELEMETRY_SCHEMA: &str = include_str!(
    "../../dist/.devtrail/schemas/charter-telemetry.schema.v0.json"
);

/// Set up a minimal DevTrail installation with both the Charter template and
/// the telemetry template + schema. Mirrors what `devtrail init` would produce
/// after fw-4.6.0 ships.
fn setup_devtrail(dir: &Path) {
    let devtrail = dir.join(".devtrail");
    std::fs::create_dir_all(devtrail.join("templates")).unwrap();
    std::fs::create_dir_all(devtrail.join("schemas")).unwrap();
    std::fs::write(devtrail.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        devtrail.join("templates").join("charter-template.md"),
        CHARTER_TEMPLATE,
    )
    .unwrap();
    std::fs::write(
        devtrail
            .join("templates")
            .join("charter-telemetry-template.yaml"),
        TELEMETRY_TEMPLATE,
    )
    .unwrap();
    std::fs::write(
        devtrail.join("schemas").join("charter-telemetry.schema.v0.json"),
        TELEMETRY_SCHEMA,
    )
    .unwrap();
}

fn create_charter(dir: &Path, title: &str) {
    Command::cargo_bin("devtrail")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg(title)
        .arg(dir.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn charter_close_requires_devtrail_installed() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn charter_close_unknown_charter_fails_clearly() {
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-99",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("CHARTER-99 not found"));
}

#[test]
fn charter_close_from_template_non_interactive_writes_telemetry_file() {
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    create_charter(dir.path(), "Test Charter");

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"))
        .stdout(predicate::str::contains("Telemetry:"));

    let telemetry_path = dir
        .path()
        .join(".devtrail/charters/CHARTER-01.telemetry.yaml");
    assert!(telemetry_path.exists(), "telemetry file should exist");

    let content = std::fs::read_to_string(&telemetry_path).unwrap();
    // Pre-fills applied.
    assert!(
        content.contains("charter_id: \"CHARTER-01\""),
        "should pre-fill charter_id, got:\n{content}"
    );
    assert!(
        content.contains("Test Charter"),
        "should pre-fill title, got:\n{content}"
    );
    assert!(
        !content.contains("YYYY-MM-DD"),
        "closed_at placeholder should be replaced with today's date, got:\n{content}"
    );
}

#[test]
fn charter_close_bumps_status_to_closed() {
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    create_charter(dir.path(), "Status Bump");

    let charter_path = dir.path().join("docs/charters/01-status-bump.md");
    let before = std::fs::read_to_string(&charter_path).unwrap();
    assert!(before.contains("status: declared"));

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&charter_path).unwrap();
    assert!(
        after.contains("status: closed"),
        "status line should be bumped, got:\n{after}"
    );
    assert!(
        !after.contains("status: declared"),
        "old declared status should be gone, got:\n{after}"
    );
}

#[test]
fn charter_close_idempotent_under_non_interactive() {
    // Running --from-template --non-interactive twice should not clobber
    // edits the user made between runs (we re-read the existing file).
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    create_charter(dir.path(), "Idempotent");

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Simulate the user editing the telemetry file.
    let telemetry_path = dir
        .path()
        .join(".devtrail/charters/CHARTER-01.telemetry.yaml");
    let edited = std::fs::read_to_string(&telemetry_path)
        .unwrap()
        .replace("ninguno", "menor");
    std::fs::write(&telemetry_path, &edited).unwrap();

    // Second run: should NOT overwrite the edit.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&telemetry_path).unwrap();
    assert!(
        after.contains("scope_changes: \"menor\""),
        "user edit should be preserved, got:\n{after}"
    );
}
